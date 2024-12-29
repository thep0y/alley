use std::io;
use std::net::Ipv4Addr;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

use tauri::Emitter;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinSet;
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::error::FluxyResult;
use crate::state::{
    app_state::AppState,
    peer::PairStatus,
    transfer::{TransferStatus, TransferTask, TransferType},
};

use super::protocol::{Message, MessageType};

pub struct TcpServer {
    state: Arc<AppState>,
    listener: TcpListener,
    cancel_token: CancellationToken,
    tasks: JoinSet<()>,
}

impl TcpServer {
    pub async fn new(state: Arc<AppState>) -> FluxyResult<Self> {
        trace!("Creating new TCP server instance");
        let listener = TcpListener::bind((Ipv4Addr::UNSPECIFIED, 0))
            .await
            .map_err(|e| {
                error!(error = ?e, "Failed to bind TCP listener");
                e
            })?;
        let local_port = listener
            .local_addr()
            .map_err(|e| {
                error!(error = ?e, "Failed to get local address");
                e
            })?
            .port();
        info!(
            port = local_port,
            addr = ?listener.local_addr().unwrap(),
            "TCP server successfully created and bound"
        );

        state.server_port.store(local_port, Ordering::SeqCst);
        debug!("Server port stored in application state");

        Ok(Self {
            state,
            listener,
            cancel_token: CancellationToken::new(),
            tasks: JoinSet::new(),
        })
    }

    pub async fn start(&mut self) -> io::Result<()> {
        info!(
            addr = ?self.listener.local_addr()?,
            "TCP server starting up"
        );

        loop {
            tokio::select! {
                result = self.listener.accept() => {
                    match result {
                        Ok((stream, addr)) => {
                            debug!(
                                peer_addr = ?addr,
                                local_addr = ?stream.local_addr(),
                                "Accepted new TCP connection"
                            );
                            let state = self.state.clone();
                            let cancel_token = self.cancel_token.clone();
                            self.tasks.spawn(async move {
                                if let Err(e) = Self::handle_connection(stream, state, cancel_token).await {
                                    error!(
                                        error = ?e,
                                        peer_addr = ?addr,
                                        "Connection handler encountered an error"
                                    );
                                }
                            });
                        }
                        Err(e) => {
                            error!(
                                error = ?e,
                                "Failed to accept incoming connection"
                            );
                        }
                    }
                }
                _ = self.cancel_token.cancelled() => {
                    info!("Received cancellation signal, initiating server shutdown");
                    break;
                }
            }
        }

        Ok(())
    }

    pub async fn stop(&mut self) -> FluxyResult<()> {
        info!("Initiating TCP server shutdown");

        trace!("Triggering cancellation token");
        // 触发取消信号
        self.cancel_token.cancel();

        // 设置超时时间，防止任务无法退出
        let timeout_duration = Duration::from_secs(5);
        debug!(timeout_secs = 5, "Waiting for tasks to complete");

        let mut completed_tasks = 0;
        while let Some(task) = timeout(timeout_duration, self.tasks.join_next()).await? {
            match task {
                Ok(_) => {
                    completed_tasks += 1;
                    trace!(completed = completed_tasks, "Task completed successfully");
                }
                Err(e) => {
                    error!(
                        error = ?e,
                        "Task failed during shutdown"
                    );
                }
            }
        }

        info!(
            completed_tasks = completed_tasks,
            "Server shutdown completed"
        );
        Ok(())
    }

    async fn handle_connection(
        mut stream: TcpStream,
        state: Arc<AppState>,
        cancel_token: CancellationToken,
    ) -> io::Result<()> {
        let peer_addr = stream.peer_addr()?;
        debug!(peer_addr = ?peer_addr, "Starting connection handler");

        let (mut reader, mut writer) = stream.split();

        loop {
            tokio::select! {
                result = Message::read_from(&mut reader) => {
                    match result {
                        Ok(Some(message)) => {
                            trace!(
                                message_type = ?message.message_type,
                                source_id = ?message.source_id,
                                "Received message"
                            );

                            if let Err(e) = Self::handle_message(&message, state.clone()).await {
                                error!(
                                    error = ?e,
                                    message_type = ?message.message_type,
                                    source_id = ?message.source_id,
                                    "Message handling failed"
                                );
                            }
                        }
                        Ok(None) => {
                            debug!(peer_addr = ?peer_addr, "Connection closed by peer");
                            break;
                        }
                        Err(e) => {
                            error!(
                                error = ?e,
                                peer_addr = ?peer_addr,
                                "Failed to read message from stream"
                            );
                            break;
                        }
                    }
                }
                _ = cancel_token.cancelled() => {
                    info!(peer_addr = ?peer_addr, "Gracefully shutting down connection");
                    // 收到取消信号，优雅地关闭连接
                    if let Err(e) = writer.shutdown().await {
                        warn!(
                            error = ?e,
                            peer_addr = ?peer_addr,
                            "Error during connection shutdown"
                        );
                    }
                    break;
                }
            }
        }
        Ok(())
    }

    async fn handle_message(message: &Message, state: Arc<AppState>) -> io::Result<()> {
        trace!(
            message_type = ?message.message_type,
            source_id = ?message.source_id,
            "Processing message"
        );

        match &message.message_type {
            MessageType::PairRequest { .. } => Self::handle_pair_request(message, state).await,
            MessageType::PairResponse { accepted, .. } => {
                Self::handle_pair_response(message, *accepted, state).await
            }
            MessageType::TransferRequest {
                transfer_id,
                file_name,
                file_size,
            } => {
                Self::handle_transfer_request(message, transfer_id, file_name, *file_size, state)
                    .await
            }
            MessageType::FileData {
                transfer_id,
                chunk_index,
                data,
            } => Self::handle_file_data(transfer_id, *chunk_index, data, state).await,
            _ => {
                debug!(
                    message_type = ?message.message_type,
                    source_id = ?message.source_id,
                    "Received unhandled message type"
                );
                Ok(())
            }
        }
    }

    async fn handle_pair_request(message: &Message, state: Arc<AppState>) -> io::Result<()> {
        debug!(
            source_id = ?message.source_id,
            "Processing pair request"
        );

        state
            .update_pair_status(message.source_id.clone(), PairStatus::RequestReceived)
            .await;

        if let Err(e) = state
            .app_handle
            .emit("pair-request-received", &message.source_id)
        {
            error!(
                error = ?e,
                source_id = ?message.source_id,
                "Failed to emit pair-request-received event"
            );
        }

        info!(
            source_id = ?message.source_id,
            "Pair request processed successfully"
        );
        Ok(())
    }

    async fn handle_pair_response(
        message: &Message,
        accepted: bool,
        state: Arc<AppState>,
    ) -> io::Result<()> {
        debug!(
            source_id = ?message.source_id,
            accepted = accepted,
            "Processing pair response"
        );

        let status = if accepted {
            PairStatus::Paired
        } else {
            PairStatus::Rejected
        };

        state
            .update_pair_status(message.source_id.clone(), status.clone())
            .await;

        info!(
            source_id = ?message.source_id,
            accepted = accepted,
            status = ?status,
            "Pair response processed successfully"
        );
        Ok(())
    }

    async fn handle_transfer_request(
        message: &Message,
        transfer_id: &str,
        file_name: &str,
        file_size: u64,
        state: Arc<AppState>,
    ) -> io::Result<()> {
        debug!(
            source_id = ?message.source_id,
            transfer_id = transfer_id,
            file_name = file_name,
            file_size = file_size,
            "Processing transfer request"
        );

        let task = TransferTask {
            id: Uuid::new_v4().to_string(),
            peer_id: message.source_id.clone(),
            transfer_type: TransferType::File {
                path: file_name.to_string(),
            },
            name: file_name.to_string(),
            size: file_size,
            progress: 0.0,
            status: TransferStatus::Pending,
        };

        state.create_transfer(task.clone()).await;

        info!(
            transfer_id = ?transfer_id,
            source_id = ?message.source_id,
            file_name = file_name,
            file_size = file_size,
            task_id = ?task.id,
            "Transfer request processed successfully"
        );
        Ok(())
    }

    async fn handle_file_data(
        transfer_id: &str,
        chunk_index: u32,
        data: &[u8],
        state: Arc<AppState>,
    ) -> io::Result<()> {
        trace!(
            transfer_id = transfer_id,
            chunk_index = chunk_index,
            chunk_size = data.len(),
            "Processing file data chunk"
        );

        if let Some(task) = state.transfers.write().await.get_mut(transfer_id) {
            let old_progress = task.progress;
            task.progress = (chunk_index as f32 * 1024.0) / task.size as f32;

            debug!(
                transfer_id = transfer_id,
                chunk_index = chunk_index,
                old_progress = old_progress,
                new_progress = task.progress,
                "Updating transfer progress"
            );

            state.notify_transfer_update(transfer_id).await;

            // 处理文件数据...
        } else {
            warn!(
                transfer_id = transfer_id,
                "Received file data for unknown transfer"
            );
        }
        Ok(())
    }
}
