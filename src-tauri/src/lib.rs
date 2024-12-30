use std::sync::Arc;

use tauri::{AppHandle, Manager};
use tokio::sync::Mutex;

use crate::commands::discovery::{
    start_broadcasting, start_listening, stop_broadcasting, stop_listening,
};
use crate::commands::peer_commands::{
    clear_peers, get_peers, respond_pair_request, send_pair_request,
};
use crate::commands::state::get_self_id;
use crate::commands::tcp::start_server;
use crate::commands::transfer_commands::send_text;
use crate::discovery::{broadcaster::Broadcaster, create_socket, listener::Listener};
use crate::error::FluxyResult;
use crate::log::setup_logging;
use crate::network::tcp_server::TcpServer;
use crate::state::app_state::AppState;

mod commands;
mod discovery;
mod error;
mod lazy;
mod log;
mod network;
mod os;
mod state;
mod transfer;

#[macro_use]
extern crate tracing;

#[cfg(desktop)]
#[tauri::command]
fn show_window(app: AppHandle, label: &str) -> FluxyResult<()> {
    debug!(label = label, "Showing window");

    if let Some(window) = app.get_webview_window(label) {
        trace!(label = label, "Window found for label");
        window.show().map_err(|e| {
            error!(error = ?e, "Failed to show window");
            e
        })?;
        window.set_focus().map_err(|e| {
            error!(error = ?e, "Failed to set window focus");
            e
        })?;
    } else {
        warn!(label = label, "No window found with label");
    }

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() {
    setup_logging();

    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.handle().clone();

            tokio::spawn(async move {
                let socket = match create_socket().await {
                    Ok(socket) => Arc::new(socket),
                    Err(e) => {
                        error!("Failed to create socket: {:?}", e);
                        return;
                    }
                };

                let app_state = Arc::new(AppState::new(app_handle.clone()));

                let tcp_server = match TcpServer::new(app_state.clone()).await {
                    Ok(s) => s,
                    Err(e) => {
                        error!("Failed to create tcp server: {:?}", e);
                        return;
                    }
                };
                let tcp_server = Arc::new(Mutex::new(tcp_server));
                app_handle.manage(tcp_server);

                let broadcaster = match Broadcaster::new(socket.clone(), app_state.clone()) {
                    Ok(b) => Arc::new(b),
                    Err(e) => {
                        error!("Failed to create broadcaster: {:?}", e);
                        return;
                    }
                };
                app_handle.manage(broadcaster);

                let listener = Arc::new(Listener::new(socket.clone(), app_state.clone()));
                app_handle.manage(listener);

                app_handle.manage(app_state);
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            #[cfg(desktop)]
            show_window,
            get_self_id,
            get_peers,
            clear_peers,
            send_pair_request,
            respond_pair_request,
            start_server,
            start_broadcasting,
            start_listening,
            stop_broadcasting,
            stop_listening,
            send_text
        ])
        .run(tauri::generate_context!())
        .map_err(|e| {
            error!(error = ?e, "Failed to build app");
            e
        })
        .expect("Error while running tauri application");
}
