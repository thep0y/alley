use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub id: String,
    pub hostname: String,
    pub addr: SocketAddr,
    pub last_seen: u64,
    pub version: String,
    pub status: PeerStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PeerStatus {
    Online,
    Offline,
    Busy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PairStatus {
    None,
    Requested,
    RequestReceived,
    Paired,
    Rejected,
}
