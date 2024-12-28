use std::sync::Arc;

use tauri::State;

use crate::{
    discovery::broadcaster::Broadcaster,
    network::tcp_client::TcpClient,
    state::{app_state::AppState, peer::PeerInfo},
};

#[tauri::command]
pub async fn get_peers(state: State<'_, Arc<AppState>>) -> Result<Vec<PeerInfo>, String> {
    let peers = state.peers.read().await;
    let peers = peers.values().map(|value| value.clone()).collect();
    Ok(peers)
}

#[tauri::command]
pub async fn send_pair_request(
    state: State<'_, Arc<AppState>>,
    target_id: String,
) -> Result<(), String> {
    let client = TcpClient::new(state.inner().clone());
    let peer = {
        let peers = state.peers.read().await;
        peers.get(&target_id).ok_or("Peer not found")?.clone()
    };

    let mut stream = client
        .connect(&peer.addr.to_string())
        .await
        .map_err(|e| e.to_string())?;
    client
        .send_pair_request(&mut stream, &target_id)
        .await
        .map_err(|e| e.to_string())?;

    state
        .update_pair_status(target_id, crate::state::peer::PairStatus::Requested)
        .await;
    Ok(())
}

#[tauri::command]
pub async fn respond_pair_request(
    state: State<'_, Arc<AppState>>,
    target_id: String,
    accepted: bool,
) -> Result<(), String> {
    let client = TcpClient::new(state.inner().clone());
    let peer = {
        let peers = state.peers.read().await;
        peers.get(&target_id).ok_or("Peer not found")?.clone()
    };

    let mut stream = client
        .connect(&peer.addr.to_string())
        .await
        .map_err(|e| e.to_string())?;
    client
        .send_pair_response(&mut stream, &target_id, accepted)
        .await
        .map_err(|e| e.to_string())?;

    let status = if accepted {
        crate::state::peer::PairStatus::Paired
    } else {
        crate::state::peer::PairStatus::Rejected
    };
    state.update_pair_status(target_id, status).await;
    Ok(())
}

#[tauri::command]
pub async fn accept_pair_request(
    state: State<'_, Arc<AppState>>,
    broadcaster: State<'_, Arc<Broadcaster>>,
    target_id: String,
) -> Result<(), String> {
    // 停止组播
    broadcaster.stop_broadcasting();

    // 更新配对状态
    state.set_paired(true);
    state
        .update_pair_status(target_id, crate::state::peer::PairStatus::Paired)
        .await;

    Ok(())
}
