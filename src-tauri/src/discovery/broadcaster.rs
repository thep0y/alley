use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{SystemTime, UNIX_EPOCH},
};

use tokio::{
    net::UdpSocket,
    sync::{Mutex, Notify},
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
    hostname: String,
    state: Arc<AppState>,
    shutdown: Arc<Mutex<Notify>>,
    is_broadcasting: AtomicBool,
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
            state,
            hostname,
            shutdown: Arc::new(Mutex::new(Notify::new())),
            is_broadcasting: AtomicBool::new(false),
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
            node_id: self.state.get_self_id().to_owned(),
            hostname: self.hostname.clone(),
            timestamp,
            protocol_version: "1.0".to_string(),
            server_port: self.state.get_server_port(),
            os_info: self.state.get_os_info().clone(),
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
        if self.is_broadcasting.swap(true, Ordering::SeqCst) {
            info!("Broadcasting is already running");
            return Ok(());
        }

        info!("Starting broadcasting");
        let mut interval = interval(Duration::from_secs(1));

        loop {
            let shutdown = self.shutdown.lock().await;
            tokio::select! {
                _ = shutdown.notified() => {
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

        self.is_broadcasting.store(false, Ordering::SeqCst);
        info!("Broadcasting stopped");
        Ok(())
    }

    pub async fn reset_shutdown(&self) {
        let mut shutdown = self.shutdown.lock().await;
        *shutdown = Notify::new();
    }

    pub async fn stop_broadcasting(&self) {
        info!("Stopping broadcasting");
        let shutdown = self.shutdown.lock().await;
        shutdown.notify_one();
    }
}
