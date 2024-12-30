use std::{io, path::Path, sync::Arc};

use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt, BufWriter},
    net::TcpStream,
};

use crate::{
    network::protocol::{Message, MessageType},
    state::app_state::AppState,
};

pub struct FileReceiver {
    state: Arc<AppState>,
}

impl FileReceiver {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }

    pub async fn receive_file(
        &self,
        mut stream: TcpStream,
        transfer_id: &str,
        path: &Path,
    ) -> io::Result<()> {
        let file = File::create(path).await?;
        let mut writer = BufWriter::new(file);

        loop {
            let mut buffer = [0u8; 1024];
            let n = stream.read(&mut buffer).await?;
            if n == 0 {
                break;
            }

            let message = Message::read_from(&mut &buffer[..n]).await?;
            if let Some(Message {
                message_type: MessageType::FileData { data, .. },
                ..
            }) = message
            {
                writer.write_all(&data).await?;
            } else {
                // 收到非文件数据消息，可能是传输取消或其他消息
                break;
            }
        }

        writer.flush().await?;

        // 更新传输状态为完成
        self.state
            .update_transfer_status(
                transfer_id,
                crate::state::transfer::TransferStatus::Completed,
            )
            .await;

        Ok(())
    }
}
