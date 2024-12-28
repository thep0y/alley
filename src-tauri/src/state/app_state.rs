use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use tauri::Emitter;
use tokio::sync::RwLock;
use uuid::Uuid;

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
    pub is_paired: AtomicBool, // 新增字段
}

impl AppState {
    pub fn new(app_handle: tauri::AppHandle) -> Self {
        // 在初始化时生成 self_id
        let self_id = Uuid::new_v4().to_string();

        Self {
            self_id,
            peers: Arc::new(RwLock::new(HashMap::new())),
            transfers: Arc::new(RwLock::new(HashMap::new())),
            pair_states: Arc::new(RwLock::new(HashMap::new())),
            app_handle,
            is_paired: AtomicBool::new(false), // 默认未配对
        }
    }

    pub fn is_paired(&self) -> bool {
        self.is_paired.load(Ordering::Relaxed)
    }

    pub fn get_self_id(&self) -> &str {
        &self.self_id
    }

    pub fn set_paired(&self, paired: bool) {
        self.is_paired.store(paired, Ordering::Relaxed);
    }

    // 节点状态管理
    pub async fn update_peer(&self, peer_id: String, peer_info: PeerInfo) {
        let mut peers = self.peers.write().await;
        peers.insert(peer_id, peer_info);
        self.notify_peer_update().await;
    }

    pub async fn remove_peer(&self, peer_id: &str) {
        let mut peers = self.peers.write().await;
        peers.remove(peer_id);
        self.notify_peer_update().await;
    }

    pub async fn cleanup_stale_peers(&self, timeout_secs: u64) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut peers = self.peers.write().await;
        peers.retain(|_, peer| now - peer.last_seen <= timeout_secs);
        self.notify_peer_update().await;
    }

    // 配对状态管理
    pub async fn update_pair_status(&self, peer_id: String, status: PairStatus) {
        let mut pair_states = self.pair_states.write().await;
        pair_states.insert(peer_id.clone(), status);
        self.notify_pair_update(&peer_id).await;
    }

    // 传输任务管理
    pub async fn create_transfer(&self, task: TransferTask) -> String {
        let task_id = uuid::Uuid::new_v4().to_string();
        let mut transfers = self.transfers.write().await;
        transfers.insert(task_id.clone(), task);
        self.notify_transfer_update(&task_id).await;
        task_id
    }

    pub async fn update_transfer_status(&self, task_id: &str, status: TransferStatus) {
        let mut transfers = self.transfers.write().await;
        if let Some(task) = transfers.get_mut(task_id) {
            task.status = status;
            self.notify_transfer_update(task_id).await;
        }
    }

    pub async fn update_transfer_progress(&self, task_id: &str, progress: f32) {
        let mut transfers = self.transfers.write().await;
        if let Some(task) = transfers.get_mut(task_id) {
            task.progress = progress;
            self.notify_transfer_update(task_id).await;
        }
    }

    // 前端通知方法
    pub async fn notify_peer_update(&self) {
        let peers = self.peers.read().await;
        let peer_list: Vec<_> = peers.values().cloned().collect();
        self.app_handle.emit("peer-update", peer_list).unwrap();
    }

    async fn notify_pair_update(&self, peer_id: &str) {
        let pair_states = self.pair_states.read().await;
        if let Some(status) = pair_states.get(peer_id) {
            self.app_handle
                .emit("pair-update", (peer_id, status))
                .unwrap();
        }
    }

    pub async fn notify_transfer_update(&self, task_id: &str) {
        let transfers = self.transfers.read().await;
        if let Some(task) = transfers.get(task_id) {
            self.app_handle.emit("transfer-update", task).unwrap();
        }
    }
}
