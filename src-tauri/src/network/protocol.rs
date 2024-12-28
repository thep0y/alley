use std::io;

use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    // 配对相关消息
    PairRequest {
        hostname: String, // 添加主机名便于用户确认
    },
    PairResponse {
        accepted: bool,
        hostname: String,
    },

    // 其他消息类型保持不变...
    TransferRequest {
        transfer_id: String,
        file_name: String,
        file_size: u64,
    },
    TransferAccept {
        transfer_id: String,
    },
    TransferReject {
        transfer_id: String,
    },
    TransferCancel {
        transfer_id: String,
    },

    FileData {
        transfer_id: String,
        chunk_index: u32,
        data: Vec<u8>,
    },
    TextMessage {
        transfer_id: String,
        content: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,        // 消息唯一标识
    pub source_id: String, // 发送方节点 ID
    pub target_id: String, // 接收方节点 ID
    pub message_type: MessageType,
}

impl Message {
    // 序列化消息到字节流
    pub async fn write_to<W: AsyncWriteExt + Unpin>(self, writer: &mut W) -> io::Result<()> {
        let data = serde_json::to_vec(&self)?;
        let len = data.len() as u32;
        writer.write_u32_le(len).await?;
        writer.write_all(&data).await?;
        writer.flush().await?;
        Ok(())
    }

    // 从字节流读取消息
    pub async fn read_from<R: AsyncReadExt + Unpin>(reader: &mut R) -> io::Result<Option<Self>> {
        let len = match reader.read_u32_le().await {
            Ok(len) => len,
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
            Err(e) => return Err(e),
        };

        let mut buffer = vec![0; len as usize];
        reader.read_exact(&mut buffer).await?;

        let message = serde_json::from_slice(&buffer)?;
        Ok(Some(message))
    }
}
