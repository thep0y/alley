use std::{io, sync::Arc};

use tokio::{io::AsyncReadExt, net::TcpStream};

use crate::{
    network::protocol::{Message, MessageType},
    state::app_state::AppState,
};

pub struct TcpClient {
    state: Arc<AppState>,
}

impl TcpClient {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }

    pub async fn connect(&self, addr: &str) -> io::Result<TcpStream> {
        let stream = TcpStream::connect(addr).await?;
        Ok(stream)
    }

    pub async fn send_pair_request(
        &self,
        stream: &mut TcpStream,
        target_id: &str,
    ) -> io::Result<()> {
        let message = Message {
            id: uuid::Uuid::new_v4().to_string(),
            source_id: "self_id".to_string(),
            target_id: target_id.to_string(),
            message_type: MessageType::PairRequest {
                hostname: hostname::get()?
                    .into_string()
                    .unwrap_or_else(|_| "unknown".to_string()),
            },
        };
        message.write_to(stream).await
    }

    pub async fn send_pair_response(
        &self,
        stream: &mut TcpStream,
        target_id: &str,
        accepted: bool,
    ) -> io::Result<()> {
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
        message.write_to(stream).await
    }

    pub async fn send_file(
        &self,
        stream: &mut TcpStream,
        target_id: &str,
        transfer_id: &str,
        path: &str,
    ) -> io::Result<()> {
        const CHUNK_SIZE: usize = 1024 * 64; // 64KB chunks
        let file = tokio::fs::File::open(path).await?;
        let metadata = file.metadata().await?;
        let total_size = metadata.len();

        // 发送传输请求
        let message = Message {
            id: uuid::Uuid::new_v4().to_string(),
            source_id: "self_id".to_string(),
            target_id: target_id.to_string(),
            message_type: MessageType::TransferRequest {
                transfer_id: transfer_id.to_string(),
                file_name: path.to_string(),
                file_size: total_size,
            },
        };
        message.write_to(stream).await?;

        let mut file = tokio::io::BufReader::new(file);
        let mut buffer = vec![0; CHUNK_SIZE];
        let mut chunk_index = 0;

        loop {
            let n = file.read(&mut buffer).await?;
            if n == 0 {
                break;
            }

            let message = Message {
                id: uuid::Uuid::new_v4().to_string(),
                source_id: "self_id".to_string(),
                target_id: target_id.to_string(),
                message_type: MessageType::FileData {
                    transfer_id: transfer_id.to_string(),
                    chunk_index,
                    data: buffer[..n].to_vec(),
                },
            };
            message.write_to(stream).await?;
            chunk_index += 1;
        }

        Ok(())
    }

    pub async fn send_text(
        &self,
        stream: &mut TcpStream,
        target_id: &str,
        transfer_id: &str,
        content: &str,
    ) -> io::Result<()> {
        let message = Message {
            id: uuid::Uuid::new_v4().to_string(),
            source_id: self.state.get_self_id().to_string(),
            target_id: target_id.to_string(),
            message_type: MessageType::TextMessage {
                transfer_id: transfer_id.to_string(),
                content: content.to_string(),
            },
        };
        message.write_to(stream).await
    }

    pub async fn send_transfer_cancel(
        &self,
        stream: &mut TcpStream,
        target_id: &str,
        transfer_id: &str,
    ) -> io::Result<()> {
        let message = Message {
            id: uuid::Uuid::new_v4().to_string(),
            source_id: self.state.get_self_id().to_string(),
            target_id: target_id.to_string(),
            message_type: MessageType::TransferCancel {
                transfer_id: transfer_id.to_string(),
            },
        };
        message.write_to(stream).await
    }
}
