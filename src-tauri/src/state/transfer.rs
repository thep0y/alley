use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferTask {
    pub id: String,
    pub peer_id: String,
    pub transfer_type: TransferType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TransferType {
    File {
        path: String,
        name: String,
        size: u64,
        progress: f32,
        status: TransferStatus,
    },
    Directory {
        path: String,
        total_files: u32,
    },
    Text {
        content: String,
    },
}

impl TransferType {
    pub fn update_progress(&mut self, new_progress: f32) {
        if let TransferType::File {
            ref mut progress, ..
        } = self
        {
            *progress = new_progress;
        }
    }

    pub fn update_status(&mut self, new_status: TransferStatus) {
        if let TransferType::File { ref mut status, .. } = self {
            *status = new_status;
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransferStatus {
    Pending,
    Transferring,
    Completed,
    Failed(String),
    Cancelled,
}
