use std::sync::Arc;

use tauri::State;

use crate::{
    discovery::{broadcaster::Broadcaster, listener::Listener},
    error::FluxyResult,
};

#[tauri::command]
pub async fn start_broadcasting(state: State<'_, Arc<Broadcaster>>) -> FluxyResult<()> {
    let broadcaster = state.inner().clone();
    broadcaster.reset_shutdown().await;
    tokio::spawn(async move { broadcaster.start_broadcasting().await });
    Ok(())
}

#[tauri::command]
pub async fn start_listening(state: State<'_, Arc<Listener>>) -> FluxyResult<()> {
    let listener = state.inner().clone();
    listener.reset_shutdown().await;
    tokio::spawn(async move { listener.start_listening().await });
    Ok(())
}

#[tauri::command]
pub async fn stop_broadcasting(state: State<'_, Arc<Broadcaster>>) -> FluxyResult<()> {
    state.stop_broadcasting().await;
    Ok(())
}

#[tauri::command]
pub async fn stop_listening(state: State<'_, Arc<Listener>>) -> FluxyResult<()> {
    state.stop_listening().await;
    Ok(())
}
