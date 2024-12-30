use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, AtomicU16, Ordering},
        Arc,
    },
};

use tauri::Emitter;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{error::FluxyResult, os::OsInformation};

use super::{
    peer::{PairStatus, PeerInfo},
    transfer::{TransferStatus, TransferTask},
};

pub struct AppState {
    // 当前设备的唯一标识符
    self_id: String,
    // 对等节点信息
    pub peers: Arc<RwLock<HashMap<String, PeerInfo>>>,
    // 传输任务信息
    pub transfers: Arc<RwLock<HashMap<String, TransferTask>>>,
    // 配对状态
    pub pair_states: Arc<RwLock<HashMap<String, PairStatus>>>,
    // Tauri 应用句柄，用于发送事件到前端
    pub app_handle: tauri::AppHandle,
    // 是否已配对
    pub is_paired: AtomicBool,
    pub server_port: AtomicU16,
    pub os_info: OsInformation,
}

impl AppState {
    pub fn new(app_handle: tauri::AppHandle) -> Self {
        // 在初始化时生成 self_id
        let self_id = Uuid::new_v4().to_string();
        info!(self_id = %self_id, "Initializing new AppState");

        let state = Self {
            self_id,
            peers: Arc::new(RwLock::new(HashMap::new())),
            transfers: Arc::new(RwLock::new(HashMap::new())),
            pair_states: Arc::new(RwLock::new(HashMap::new())),
            app_handle,
            is_paired: AtomicBool::new(false),
            server_port: 0.into(),
            os_info: OsInformation::new(),
        };

        debug!("AppState initialized successfully");
        state
    }

    pub fn get_server_port(&self) -> u16 {
        let port = self.server_port.load(Ordering::SeqCst);
        trace!(port = port, "Retrieved server port");
        port
    }

    pub fn get_os_info(&self) -> OsInformation {
        trace!("Retrieving OS information");
        self.os_info.clone()
    }

    pub fn is_paired(&self) -> bool {
        let paired = self.is_paired.load(Ordering::Relaxed);
        trace!(paired = paired, "Retrieved paired status");
        paired
    }

    pub fn get_self_id(&self) -> &str {
        trace!(self_id = %self.self_id, "Retrieved self ID");
        &self.self_id
    }

    pub fn set_paired(&self, paired: bool) {
        info!(paired = paired, "Setting paired status");
        self.is_paired.store(paired, Ordering::Relaxed);
    }

    // 节点状态管理
    pub async fn update_peer(&self, peer_id: String, peer_info: PeerInfo) {
        info!(peer_id = %peer_id, "Updating peer information");
        let mut peers = self.peers.write().await;
        peers.insert(peer_id, peer_info);
        debug!("Peer information updated successfully");

        if let Err(e) = self.notify_peer_update().await {
            error!(error = ?e, "Failed to notify peer update");
        }
    }

    pub async fn remove_peer(&self, peer_id: &str) {
        info!(peer_id = %peer_id, "Removing peer");
        let mut peers = self.peers.write().await;
        if peers.remove(peer_id).is_some() {
            debug!(peer_id = %peer_id, "Peer removed successfully");
        } else {
            warn!(peer_id = %peer_id, "Attempted to remove non-existent peer");
        }

        if let Err(e) = self.notify_peer_update().await {
            error!(error = ?e, "Failed to notify peer removal");
        }
    }

    pub async fn cleanup_stale_peers(&self, timeout_secs: u64) {
        info!(timeout_secs = timeout_secs, "Starting stale peer cleanup");
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_else(|e| {
                error!(error = ?e, "Failed to get system time");
                std::time::Duration::from_secs(0)
            })
            .as_secs();

        let mut peers = self.peers.write().await;
        let initial_count = peers.len();

        peers.retain(|peer_id, peer| {
            let is_active = now - peer.last_seen <= timeout_secs;
            if !is_active {
                debug!(peer_id = %peer_id, "Removing stale peer");
            }
            is_active
        });

        let removed_count = initial_count - peers.len();
        info!(
            removed_count = removed_count,
            remaining_count = peers.len(),
            "Completed stale peer cleanup"
        );

        if let Err(e) = self.notify_peer_update().await {
            error!(error = ?e, "Failed to notify after stale peer cleanup");
        }
    }

    pub async fn clear_peers(&self) {
        info!("Clearing all peers");
        let mut peers = self.peers.write().await;
        let cleared_count = peers.len();
        peers.clear();
        info!(cleared_count = cleared_count, "All peers cleared");
    }

    // 配对状态管理
    pub async fn update_pair_status(&self, peer_id: String, status: PairStatus) -> FluxyResult<()> {
        info!(peer_id = %peer_id, status = ?status, "Updating pair status");
        let mut pair_states = self.pair_states.write().await;
        pair_states.insert(peer_id.clone(), status);
        drop(pair_states);
        debug!(peer_id = %peer_id, "Pair status updated successfully");
        self.notify_pair_update(&peer_id).await?;
        Ok(())
    }

    // 传输任务管理
    pub async fn create_transfer(&self, task: TransferTask) -> FluxyResult<String> {
        let task_id = uuid::Uuid::new_v4().to_string();
        info!(
            task_id = %task_id,
            peer_id = %task.peer_id,
            "Creating new transfer task"
        );

        let mut transfers = self.transfers.write().await;
        transfers.insert(task_id.clone(), task);
        debug!(task_id = %task_id, "Transfer task created successfully");
        drop(transfers);

        self.notify_transfer_update(&task_id).await?;
        Ok(task_id)
    }

    pub async fn update_transfer_status(
        &self,
        task_id: &str,
        status: TransferStatus,
    ) -> FluxyResult<()> {
        info!(task_id = %task_id, status = ?status, "Updating transfer status");
        let mut transfers = self.transfers.write().await;

        if let Some(task) = transfers.get_mut(task_id) {
            task.transfer_type.update_status(status);
            debug!(task_id = %task_id, "Transfer status updated successfully");
            self.notify_transfer_update(task_id).await?;
            Ok(())
        } else {
            let err = format!("Transfer task not found: {}", task_id);
            error!(task_id = %task_id, "Failed to update transfer status - task not found");
            Err(err.into())
        }
    }

    pub async fn update_file_transfer_progress(
        &self,
        task_id: &str,
        progress: f32,
    ) -> FluxyResult<()> {
        trace!(task_id = %task_id, progress = %progress, "Updating transfer progress");
        let mut transfers = self.transfers.write().await;

        if let Some(task) = transfers.get_mut(task_id) {
            task.transfer_type.update_progress(progress);
            trace!(task_id = %task_id, "Transfer progress updated successfully");
            self.notify_transfer_update(task_id).await?;
            Ok(())
        } else {
            let err = format!("Transfer task not found: {}", task_id);
            error!(task_id = %task_id, "Failed to update transfer progress - task not found");
            Err(err.into())
        }
    }

    // 前端通知方法
    pub async fn notify_peer_update(&self) -> FluxyResult<()> {
        trace!("Notifying peer update");
        let peers = self.peers.read().await;
        let peer_list: Vec<_> = peers.values().cloned().collect();

        self.app_handle
            .emit("peer-update", peer_list)
            .map_err(|e| {
                error!(error = ?e, "Failed to emit peer update event");
                e
            })?;

        trace!("Peer update notification sent successfully");
        Ok(())
    }

    async fn notify_pair_update(&self, peer_id: &str) -> FluxyResult<()> {
        trace!(peer_id = %peer_id, "Notifying pair update");
        let pair_states = self.pair_states.read().await;

        if let Some(status) = pair_states.get(peer_id) {
            self.app_handle
                .emit("pair-update", (peer_id, status))
                .map_err(|e| {
                    error!(
                        error = ?e,
                        peer_id = %peer_id,
                        "Failed to emit pair update event"
                    );
                    e
                })?;
            trace!(peer_id = %peer_id, "Pair update notification sent successfully");
            Ok(())
        } else {
            let err = format!("Pair state not found for peer: {}", peer_id);
            error!(peer_id = %peer_id, "Failed to notify pair update - state not found");
            Err(err.into())
        }
    }

    pub async fn notify_transfer_update(&self, task_id: &str) -> FluxyResult<()> {
        trace!(task_id = %task_id, "Notifying transfer update");
        let transfers = self.transfers.read().await;

        if let Some(task) = transfers.get(task_id) {
            self.app_handle.emit("transfer-update", task).map_err(|e| {
                error!(
                    error = ?e,
                    task_id = %task_id,
                    "Failed to emit transfer update event"
                );
                e
            })?;
            trace!(task_id = %task_id, "Transfer update notification sent successfully");
            Ok(())
        } else {
            let err = format!("Transfer task not found: {}", task_id);
            error!(task_id = %task_id, "Failed to notify transfer update - task not found");
            Err(err.into())
        }
    }
}
