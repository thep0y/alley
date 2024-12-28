use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferTask {
    pub id: String,
    pub peer_id: String,
    pub transfer_type: TransferType,
    pub name: String,
    pub size: u64,
    pub progress: f32,
    pub status: TransferStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransferType {
    File { path: String },
    Directory { path: String, total_files: u32 },
    Text { content: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransferStatus {
    Pending,
    Transferring,
    Completed,
    Failed(String),
    Cancelled,
}
