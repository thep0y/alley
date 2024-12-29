use std::sync::Arc;

use tauri::State;
use tokio::sync::Mutex;

use crate::network::tcp_server::TcpServer;
use crate::FluxyResult;

#[tauri::command]
pub async fn start_server(state: State<'_, Arc<Mutex<TcpServer>>>) -> FluxyResult<()> {
    let server = state.inner().clone();
    tokio::spawn(async move {
        let mut server = server.lock().await;
        if let Err(e) = server.start().await {
            error!("Failed to start server: {:?}", e);
        }
    });
    Ok(())
}
