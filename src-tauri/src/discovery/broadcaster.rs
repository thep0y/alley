use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use tokio::{
    net::UdpSocket,
    sync::Notify,
    time::{interval, Duration},
};

use crate::{
    discovery::{MULTICAST_ADDRESS, MULTICAST_PORT},
    error::FluxyResult,
    state::app_state::AppState,
};

use super::DiscoveryMessage;

pub struct Broadcaster {
    socket: Arc<UdpSocket>,
    node_id: String,
    hostname: String,
    shutdown: Arc<Notify>,
}

impl Broadcaster {
    pub fn new(socket: Arc<UdpSocket>, state: Arc<AppState>) -> FluxyResult<Self> {
        trace!("Creating new Broadcaster instance");

        let hostname = hostname::get()
            .map_err(|e| {
                error!(error = ?e, "Failed to get hostname");
                e
            })?
            .into_string()
            .unwrap_or_else(|_| "unknown".to_string());

        let node_id = state.get_self_id().to_owned();
        info!(
            node_id = node_id,
            hostname = hostname,
            "Broadcaster initialized"
        );

        Ok(Self {
            socket,
            node_id,
            hostname,
            shutdown: Arc::new(Notify::new()), // 默认启动组播
        })
    }

    async fn broadcast_message(&self) -> FluxyResult<()> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| {
                error!(error = ?e, "Failed to get system time");
                e
            })?
            .as_secs();

        let message = DiscoveryMessage::Announce {
            node_id: self.node_id.clone(),
            hostname: self.hostname.clone(),
            timestamp,
            version: "1.0".to_string(),
        };

        let bytes = serde_json::to_vec(&message).map_err(|e| {
            error!(error = ?e, "Failed to serialize discovery message");
            e
        })?;
        trace!(message = ?message, "Preparing to send discovery message");

        self.socket
            .send_to(&bytes, (MULTICAST_ADDRESS, MULTICAST_PORT))
            .await
            .map_err(|e| {
                error!(error = ?e, "Failed to send discovery message");
                e
            })?;

        Ok(())
    }

    pub async fn start_broadcasting(&self) -> FluxyResult<()> {
        info!("Starting broadcasting");
        let mut interval = interval(Duration::from_secs(1));

        loop {
            tokio::select! {
                _ = self.shutdown.notified() => {
                    info!("Received shutdown signal");
                    break;
                }
                _ = interval.tick() => {
                    if let Err(e) = self.broadcast_message().await {
                        error!(error = ?e, "Failed to broadcast message");
                        // 可以根据需求决定是否继续广播或中断
                    }
                }
            }
        }

        info!("Broadcasting stopped");
        Ok(())
    }

    pub fn stop_broadcasting(&self) {
        info!("Stopping broadcasting");
        self.shutdown.notify_one();
    }
}
