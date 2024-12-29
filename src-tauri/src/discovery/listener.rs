use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tokio::net::UdpSocket;
use tokio::sync::{Mutex, Notify};

use crate::error::FluxyResult;
use crate::state::app_state::AppState;
use crate::state::peer::{PeerInfo, PeerStatus};

use super::DiscoveryMessage;

pub struct Listener {
    socket: Arc<UdpSocket>,
    state: Arc<AppState>,
    shutdown: Arc<Mutex<Notify>>,
    is_listening: AtomicBool,
}

impl Listener {
    pub fn new(socket: Arc<UdpSocket>, state: Arc<AppState>) -> Self {
        Self {
            socket,
            state,
            shutdown: Arc::new(Mutex::new(Notify::new())),
            is_listening: AtomicBool::new(false),
        }
    }

    pub async fn start_listening(&self) -> FluxyResult<()> {
        if self.is_listening.swap(true, Ordering::SeqCst) {
            info!("Listening is already running");
            return Ok(());
        }

        let mut buf = [0; 1024];

        loop {
            let shutdown = self.shutdown.lock().await;
            tokio::select! {
                result = self.receive_message(&mut buf) => {
                    if let Err(e) = result {
                        error!(error = ?e, "Failed to process message");
                        return Err(e);
                    }
                }
                _ = shutdown.notified() => {
                    info!("Stopping listener");
                    break;
                }
            }
        }

        self.is_listening.store(false, Ordering::SeqCst);
        info!("Listening stopped");
        Ok(())
    }
    pub async fn reset_shutdown(&self) {
        let mut shutdown = self.shutdown.lock().await;
        *shutdown = Notify::new();
    }

    pub async fn stop_listening(&self) {
        info!("Stopping listening");
        let shutdown = self.shutdown.lock().await;
        shutdown.notify_one();
    }
    async fn receive_message(&self, buf: &mut [u8]) -> FluxyResult<()> {
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
    ) -> FluxyResult<()> {
        match message {
            DiscoveryMessage::Announce {
                node_id,
                hostname,
                timestamp,
                protocol_version,
                server_port,
                os_info,
            } => {
                if node_id == self.state.get_self_id() {
                    trace!(node_id = ?node_id, "Ignoring Announce message from self");
                    return Ok(());
                }

                debug!(node_id = node_id, hostname = hostname, server_port = server_port, timestamp = timestamp, version = protocol_version, os_info = ?os_info, "Received Announce message");

                let peer_info = PeerInfo {
                    id: node_id,
                    hostname,
                    addr: addr.ip(),
                    port: server_port,
                    last_seen: timestamp,
                    protocol_version,
                    status: PeerStatus::Online,
                    os_info,
                };

                self.update_peer_info(peer_info).await;
                self.state.notify_peer_update().await;
            }
            DiscoveryMessage::Heartbeat { node_id, timestamp } => {
                debug!(node_id = ?node_id, timestamp = timestamp, "Received Heartbeat message");
                self.update_peer_last_seen(node_id, timestamp).await;
            }
        }

        Ok(())
    }

    async fn update_peer_info(&self, peer_info: PeerInfo) {
        let node_id = peer_info.id.clone();
        let mut peers = self.state.peers.write().await;
        peers.insert(node_id.clone(), peer_info);

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
