use std::{io, path::Path, sync::Arc};

use tokio::{
    fs::File,
    io::{AsyncReadExt, BufReader},
    net::TcpStream,
};

use crate::{
    network::protocol::{Message, MessageType},
    state::{app_state::AppState, transfer::TransferTask},
};

pub struct FileSender {
    state: Arc<AppState>,
}

impl FileSender {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }

    pub async fn send_file(
        &self,
        mut stream: TcpStream,
        target_id: &str,
        transfer_id: &str,
        path: &Path,
    ) -> io::Result<()> {
        let file = File::open(path).await?;
        let metadata = file.metadata().await?;
        let file_size = metadata.len();
        let file_name = path.file_name().unwrap().to_string_lossy().to_string();

        // 发送传输请求
        let transfer_request = Message {
            id: transfer_id.to_string(),
            source_id: self.state.get_self_id().to_string(),
            target_id: target_id.to_string(),
            message_type: MessageType::TransferRequest {
                transfer_id: transfer_id.to_string(),
                file_name: file_name.clone(),
                file_size,
            },
        };
        transfer_request.write_to(&mut stream).await?;

        // 等待对方的接受或拒绝
        let mut buffer = [0u8; 1024];
        let n = stream.read(&mut buffer).await?;
        let response = Message::read_from(&mut &buffer[..n]).await?;

        if let Some(Message {
            message_type: MessageType::TransferAccept { .. },
            ..
        }) = response
        {
            // 对方接受了传输请求，开始传输文件
            let mut reader = BufReader::new(file);
            let mut chunk_index = 0;
            let mut buffer = vec![0u8; 1024 * 1024]; // 1MB chunk size

            loop {
                let n = reader.read(&mut buffer).await?;
                if n == 0 {
                    break;
                }

                let file_data = Message {
                    id: transfer_id.to_string(),
                    source_id: self.state.get_self_id().to_string(),
                    target_id: target_id.to_string(),
                    message_type: MessageType::FileData {
                        transfer_id: transfer_id.to_string(),
                        chunk_index,
                        data: buffer[..n].to_vec(),
                    },
                };
                file_data.write_to(&mut stream).await?;

                chunk_index += 1;

                // 更新传输进度
                let progress = (chunk_index * 1024 * 1024) as f32 / file_size as f32;
                self.state
                    .update_transfer_progress(transfer_id, progress)
                    .await;
            }

            // 传输完成
            self.state
                .update_transfer_status(
                    transfer_id,
                    crate::state::transfer::TransferStatus::Completed,
                )
                .await;
        } else {
            // 对方拒绝了传输请求
            self.state
                .update_transfer_status(
                    transfer_id,
                    crate::state::transfer::TransferStatus::Cancelled,
                )
                .await;
        }

        Ok(())
    }
}
