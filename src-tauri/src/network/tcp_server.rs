use std::{io, sync::Arc};

use tauri::Emitter;
use tokio::net::{TcpListener, TcpStream};
use uuid::Uuid;

use crate::state::{
    app_state::AppState,
    peer::PairStatus,
    transfer::{TransferStatus, TransferTask, TransferType},
};

use super::protocol::{Message, MessageType};

pub struct TcpServer {
    state: Arc<AppState>,
}

impl TcpServer {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }

    pub async fn start(self, addr: &str) -> io::Result<()> {
        let listener = TcpListener::bind(addr).await?;
        println!("Server listening on {}", addr);

        while let Ok((stream, addr)) = listener.accept().await {
            println!("New connection from {}", addr);
            let state = self.state.clone();
            tokio::spawn(async move {
                if let Err(e) = Self::handle_connection(stream, state).await {
                    eprintln!("Connection error: {}", e);
                }
            });
        }
        Ok(())
    }

    async fn handle_connection(mut stream: TcpStream, state: Arc<AppState>) -> io::Result<()> {
        let (mut reader, mut writer) = stream.split();

        while let Some(message) = Message::read_from(&mut reader).await? {
            match message.message_type {
                MessageType::PairRequest { .. } => {
                    state
                        .update_pair_status(message.source_id.clone(), PairStatus::RequestReceived)
                        .await;
                    // 通知前端显示配对请求
                    state
                        .app_handle
                        .emit("pair-request-received", &message.source_id)
                        .unwrap();
                }

                MessageType::PairResponse { accepted, .. } => {
                    let status = if accepted {
                        PairStatus::Paired
                    } else {
                        PairStatus::Rejected
                    };
                    state
                        .update_pair_status(message.source_id.clone(), status)
                        .await;
                }

                MessageType::TransferRequest {
                    transfer_id,
                    file_name,
                    file_size,
                } => {
                    let task = TransferTask {
                        id: Uuid::new_v4().to_string(),
                        peer_id: message.source_id.clone(),
                        transfer_type: TransferType::File {
                            path: file_name.clone(),
                        },
                        name: file_name,
                        size: file_size,
                        progress: 0.0,
                        status: TransferStatus::Pending,
                    };
                    state.create_transfer(task).await;
                }

                MessageType::FileData {
                    transfer_id,
                    chunk_index,
                    data,
                } => {
                    // 更新传输进度
                    if let Some(task) = state.transfers.write().await.get_mut(&transfer_id) {
                        task.progress = (chunk_index as f32 * 1024.0) / task.size as f32;
                        state.notify_transfer_update(&transfer_id).await;

                        // 处理文件数据...
                    }
                }

                // 处理其他消息类型...
                _ => {}
            }
        }
        Ok(())
    }
}
