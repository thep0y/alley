use super::DiscoveryMessage;
use crate::state::app_state::AppState;
use crate::state::peer::{PeerInfo, PeerStatus};
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::Notify;

pub struct Listener {
    socket: Arc<UdpSocket>,
    state: Arc<AppState>,
    shutdown: Arc<Notify>,
}

impl Listener {
    pub fn new(socket: Arc<UdpSocket>, state: Arc<AppState>) -> std::io::Result<Self> {
        Ok(Self {
            socket,
            state,
            shutdown: Arc::new(Notify::new()),
        })
    }

    pub async fn start_listening(&self) -> std::io::Result<()> {
        let mut buf = [0; 1024];

        loop {
            tokio::select! {
                result = self.receive_message(&mut buf) => {
                    if let Err(e) = result {
                        error!(error = ?e, "Failed to process message");
                        return Err(e);
                    }
                }
                _ = self.shutdown.notified() => {
                    info!("Stopping listener");
                    break;
                }
            }
        }

        Ok(())
    }

    pub fn stop_listening(&self) {
        self.shutdown.notify_one();
    }

    async fn receive_message(&self, buf: &mut [u8]) -> std::io::Result<()> {
        let (size, addr) = self.socket.recv_from(buf).await?;
        trace!(size = size, addr = ?addr, "Received data from socket");

        if let Ok(message) = serde_json::from_slice::<DiscoveryMessage>(&buf[..size]) {
            self.handle_message(message, addr).await?;
        }

        Ok(())
    }

    async fn handle_message(
        &self,
        message: DiscoveryMessage,
        addr: std::net::SocketAddr,
    ) -> std::io::Result<()> {
        match message {
            DiscoveryMessage::Announce {
                node_id,
                hostname,
                timestamp,
                version,
            } => {
                if node_id == self.state.get_self_id() {
                    trace!(node_id = ?node_id, "Ignoring Announce message from self");
                    return Ok(());
                }

                debug!(node_id = ?node_id, hostname = ?hostname, timestamp = timestamp, version = ?version, "Received Announce message");

                self.update_peer_info(node_id, hostname, addr, timestamp, version)
                    .await;
                self.state.notify_peer_update().await;
            }
            DiscoveryMessage::Heartbeat { node_id, timestamp } => {
                debug!(node_id = ?node_id, timestamp = timestamp, "Received Heartbeat message");
                self.update_peer_last_seen(node_id, timestamp).await;
            }
        }

        Ok(())
    }

    async fn update_peer_info(
        &self,
        node_id: String,
        hostname: String,
        addr: std::net::SocketAddr,
        timestamp: u64,
        version: String,
    ) {
        let mut peers = self.state.peers.write().await;
        peers.insert(
            node_id.clone(),
            PeerInfo {
                id: node_id.clone(),
                hostname,
                addr,
                last_seen: timestamp,
                version,
                status: PeerStatus::Online,
            },
        );
        info!(node_id = ?node_id, "Updated peer info from Announce message");
    }

    async fn update_peer_last_seen(&self, node_id: String, timestamp: u64) {
        let mut peers = self.state.peers.write().await;
        if let Some(peer) = peers.get_mut(&node_id) {
            peer.last_seen = timestamp;
            trace!(node_id = ?node_id, "Updated peer last seen time from Heartbeat message");
        } else {
            warn!(node_id = ?node_id, "Received Heartbeat for unknown peer");
        }
    }
}
