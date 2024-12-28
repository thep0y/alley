use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;

use crate::{network::tcp_client::TcpClient, state::app_state::AppState, transfer::FileSender};

#[derive(Debug, Serialize, Deserialize)]
pub struct Transfer {
    id: String,
    peer_id: String,
    name: String,
    size: u64,
    progress: f32,
    status: String,
}

#[tauri::command]
pub async fn get_transfers(state: State<'_, Arc<AppState>>) -> Result<Vec<Transfer>, String> {
    let transfers = state.transfers.read().await;
    let transfers = transfers
        .values()
        .map(|transfer| Transfer {
            id: transfer.id.clone(),
            peer_id: transfer.peer_id.clone(),
            name: transfer.name.clone(),
            size: transfer.size,
            progress: transfer.progress,
            status: match transfer.status {
                crate::state::transfer::TransferStatus::Pending => "Pending".to_string(),
                crate::state::transfer::TransferStatus::Transferring => "Transferring".to_string(),
                crate::state::transfer::TransferStatus::Completed => "Completed".to_string(),
                crate::state::transfer::TransferStatus::Failed(_) => "Failed".to_string(),
                crate::state::transfer::TransferStatus::Cancelled => "Cancelled".to_string(),
            },
        })
        .collect();
    Ok(transfers)
}

#[tauri::command]
pub async fn send_file(
    state: State<'_, Arc<AppState>>,
    target_id: String,
    path: PathBuf,
) -> Result<String, String> {
    let client = TcpClient::new(state.inner().clone());
    let peer = {
        let peers = state.peers.read().await;
        peers.get(&target_id).ok_or("Peer not found")?.clone()
    };

    let transfer_id = uuid::Uuid::new_v4().to_string();
    let mut stream = client
        .connect(&peer.addr.to_string())
        .await
        .map_err(|e| e.to_string())?;

    let file_sender = FileSender::new(state.inner().clone());
    file_sender
        .send_file(stream, &target_id, &transfer_id, &path)
        .await
        .map_err(|e| e.to_string())?;

    Ok(transfer_id)
}

#[tauri::command]
pub async fn cancel_transfer(
    state: State<'_, Arc<AppState>>,
    transfer_id: String,
) -> Result<(), String> {
    let transfer = {
        let transfers = state.transfers.read().await;
        transfers
            .get(&transfer_id)
            .ok_or("Transfer not found")?
            .clone()
    };

    let client = TcpClient::new(state.inner().clone());
    let peer = {
        let peers = state.peers.read().await;
        peers
            .get(&transfer.peer_id)
            .ok_or("Peer not found")?
            .clone()
    };

    let mut stream = client
        .connect(&peer.addr.to_string())
        .await
        .map_err(|e| e.to_string())?;
    client
        .send_transfer_cancel(&mut stream, &transfer.peer_id, &transfer_id)
        .await
        .map_err(|e| e.to_string())?;

    state
        .update_transfer_status(
            &transfer_id,
            crate::state::transfer::TransferStatus::Cancelled,
        )
        .await;
    Ok(())
}
