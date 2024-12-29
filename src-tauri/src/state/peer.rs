use std::net::IpAddr;

use serde::{Deserialize, Serialize};

use crate::os::OsInformation;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub id: String,
    pub hostname: String,
    pub addr: IpAddr,
    pub last_seen: u64,
    pub protocol_version: String,
    pub status: PeerStatus,
    pub port: u16,
    pub os_info: OsInformation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PeerStatus {
    Online,
    Offline,
    Busy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PairStatus {
    None,
    Requested,
    RequestReceived,
    Paired,
    Rejected,
}
