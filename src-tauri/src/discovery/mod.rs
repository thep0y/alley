use std::net::{Ipv4Addr, SocketAddrV4};

use serde::{Deserialize, Serialize};
use tokio::net::UdpSocket;

use crate::error::FluxyResult;

pub mod broadcaster;
pub mod listener;

// 发现协议的消息类型
#[derive(Debug, Serialize, Deserialize)]
pub enum DiscoveryMessage {
    // 节点宣告自己的存在
    Announce {
        node_id: String,  // 节点唯一标识
        hostname: String, // 主机名
        timestamp: u64,   // 发送时间戳
        version: String,  // 协议版本
    },
    // 心跳包
    Heartbeat {
        node_id: String,
        timestamp: u64,
    },
}

const MULTICAST_ADDRESS: Ipv4Addr = Ipv4Addr::new(226, 4, 55, 1);
const MULTICAST_PORT: u16 = 55412;

pub async fn create_socket() -> FluxyResult<UdpSocket> {
    let socket_addr = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, MULTICAST_PORT);
    let socket = UdpSocket::bind(socket_addr).await.map_err(|e| {
        error!(error = ?e, "Failed to create UdpSocket");
        e
    })?;

    socket
        .join_multicast_v4(MULTICAST_ADDRESS, Ipv4Addr::UNSPECIFIED)
        .map_err(|e| {
            error!(error = ?e, "Failed to join multicast");
            e
        })?;

    socket.set_multicast_loop_v4(true).map_err(|e| {
        error!(error = ?e, "Failed to set multicast loop");
        e
    })?;

    Ok(socket)
}
