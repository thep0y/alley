use std::sync::Arc;

use tokio::net::TcpStream;

use crate::{
    error::FluxyResult,
    network::protocol::{Message, MessageType, TransferRequestType},
    state::app_state::AppState,
};

pub struct TextSender {
    state: Arc<AppState>,
}

impl TextSender {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }

    pub async fn send_text(
        &self,
        mut stream: TcpStream,
        target_id: &str,
        transfer_id: &str,
        content: &str,
    ) -> FluxyResult<()> {
        // 发送传输请求
        let transfer_request = Message {
            id: transfer_id.to_string(),
            source_id: self.state.get_self_id().to_string(),
            target_id: target_id.to_string(),
            message_type: MessageType::TransferRequest {
                transfer_id: transfer_id.to_string(),
                transfer_request_type: TransferRequestType::Text {
                    content: content.to_string(),
                },
            },
        };
        transfer_request.write_to(&mut stream).await?;

        // 文本消息不需要对方接受, 直接传输

        self.state
            .update_transfer_status(
                transfer_id,
                crate::state::transfer::TransferStatus::Completed,
            )
            .await
    }
}
