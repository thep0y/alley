use std::{io, net::IpAddr, sync::Arc};

use tokio::{io::AsyncReadExt, net::TcpStream};

use crate::{
    network::protocol::{Message, MessageType, TransferRequestType},
    state::app_state::AppState,
};

pub struct TcpClient {
    state: Arc<AppState>,
}

impl TcpClient {
    pub fn new(state: Arc<AppState>) -> Self {
        info!("TcpClient initialized");
        Self { state }
    }

    pub async fn connect(&self, ip: IpAddr, port: u16) -> io::Result<TcpStream> {
        trace!(ip = ?ip, port = port, "Attempting to connect to server");
        let stream = TcpStream::connect((ip, port)).await.map_err(|e| {
            error!(error = ?e, "Failed to connect to server");
            e
        })?;
        info!("Connected to server successfully");
        Ok(stream)
    }

    pub async fn send_pair_request(
        &self,
        stream: &mut TcpStream,
        target_id: &str,
    ) -> io::Result<()> {
        debug!(target_id = target_id, "Sending pair request");
        let message = Message {
            id: uuid::Uuid::new_v4().to_string(),
            source_id: self.state.get_self_id().to_string(),
            target_id: target_id.to_string(),
            message_type: MessageType::PairRequest {
                hostname: hostname::get()?
                    .into_string()
                    .unwrap_or_else(|_| "unknown".to_string()),
            },
        };
        message.write_to(stream).await.map_err(|e| {
            error!(error = ?e, "Failed to send pair request");
            e
        })?;
        info!("Pair request sent successfully");
        Ok(())
    }

    pub async fn send_pair_response(
        &self,
        stream: &mut TcpStream,
        target_id: &str,
        accepted: bool,
    ) -> io::Result<()> {
        debug!(
            target_id = target_id,
            accepted = accepted,
            "Sending pair response"
        );
        let message = Message {
            id: uuid::Uuid::new_v4().to_string(),
            source_id: self.state.get_self_id().to_string(),
            target_id: target_id.to_string(),
            message_type: MessageType::PairResponse {
                accepted,
                hostname: hostname::get()?
                    .into_string()
                    .unwrap_or_else(|_| "unknown".to_string()),
            },
        };
        message.write_to(stream).await.map_err(|e| {
            error!(error = ?e, "Failed to send pair response");
            e
        })?;
        info!("Pair response sent successfully");
        Ok(())
    }

    pub async fn send_file(
        &self,
        stream: &mut TcpStream,
        target_id: &str,
        transfer_id: &str,
        path: &str,
    ) -> io::Result<()> {
        const CHUNK_SIZE: usize = 1024 * 64; // 64KB chunks
        info!(
            target_id = target_id,
            transfer_id = transfer_id,
            path = path,
            "Starting file transfer"
        );

        let file = tokio::fs::File::open(path).await.map_err(|e| {
            error!(error = ?e, "Failed to open file");
            e
        })?;
        let metadata = file.metadata().await.map_err(|e| {
            error!(error = ?e, "Failed to get file metadata");
            e
        })?;
        let total_size = metadata.len();

        // 发送传输请求
        let message = Message {
            id: uuid::Uuid::new_v4().to_string(),
            source_id: "self_id".to_string(),
            target_id: target_id.to_string(),
            message_type: MessageType::TransferRequest {
                transfer_id: transfer_id.to_string(),
                transfer_request_type: TransferRequestType::File {
                    file_name: path.to_string(),
                    file_size: total_size,
                },
            },
        };
        message.write_to(stream).await.map_err(|e| {
            error!(error = ?e, "Failed to send transfer request");
            e
        })?;

        let mut file = tokio::io::BufReader::new(file);
        let mut buffer = vec![0; CHUNK_SIZE];
        let mut chunk_index = 0;

        loop {
            let n = file.read(&mut buffer).await.map_err(|e| {
                error!(error = ?e, "Failed to read file chunk");
                e
            })?;
            if n == 0 {
                break;
            }

            let message = Message {
                id: uuid::Uuid::new_v4().to_string(),
                source_id: self.state.get_self_id().to_string(),
                target_id: target_id.to_string(),
                message_type: MessageType::FileData {
                    transfer_id: transfer_id.to_string(),
                    chunk_index,
                    data: buffer[..n].to_vec(),
                },
            };
            message.write_to(stream).await.map_err(|e| {
                error!(error = ?e, "Failed to send file chunk");
                e
            })?;
            chunk_index += 1;
        }

        info!("File transfer completed successfully");
        Ok(())
    }

    pub async fn send_text(
        &self,
        stream: &mut TcpStream,
        target_id: &str,
        transfer_id: &str,
        content: &str,
    ) -> io::Result<()> {
        debug!(
            target_id = target_id,
            transfer_id = transfer_id,
            "Sending text message"
        );
        let message = Message {
            id: uuid::Uuid::new_v4().to_string(),
            source_id: self.state.get_self_id().to_string(),
            target_id: target_id.to_string(),
            message_type: MessageType::TextMessage {
                transfer_id: transfer_id.to_string(),
                content: content.to_string(),
            },
        };
        message.write_to(stream).await.map_err(|e| {
            error!(error = ?e, "Failed to send text message");
            e
        })?;
        info!("Text message sent successfully");
        Ok(())
    }

    pub async fn send_transfer_cancel(
        &self,
        stream: &mut TcpStream,
        target_id: &str,
        transfer_id: &str,
    ) -> io::Result<()> {
        debug!(
            target_id = target_id,
            transfer_id = transfer_id,
            "Sending transfer cancel"
        );
        let message = Message {
            id: uuid::Uuid::new_v4().to_string(),
            source_id: self.state.get_self_id().to_string(),
            target_id: target_id.to_string(),
            message_type: MessageType::TransferCancel {
                transfer_id: transfer_id.to_string(),
            },
        };
        message.write_to(stream).await.map_err(|e| {
            error!(error = ?e, "Failed to send transfer cancel");
            e
        })?;
        info!("Transfer cancel sent successfully");
        Ok(())
    }
}
