use std::sync::Arc;

use tauri::{AppHandle, Manager};

use crate::commands::peer_commands::get_peers;
use crate::discovery::{broadcaster::Broadcaster, create_socket, listener::Listener};
use crate::error::FluxyResult;
use crate::log::setup_logging;
use crate::state::app_state::AppState;

mod commands;
mod discovery;
mod error;
mod lazy;
mod log;
mod network;
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
        .plugin(tauri_plugin_os::init())
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

                // 启动组播
                let broadcaster = match Broadcaster::new(socket.clone(), app_state.clone()) {
                    Ok(b) => Arc::new(b),
                    Err(e) => {
                        error!("Failed to create broadcaster: {:?}", e);
                        return;
                    }
                };

                let broadcaster_clone: Arc<Broadcaster> = broadcaster.clone();
                tokio::spawn(async move {
                    if let Err(e) = broadcaster_clone.start_broadcasting().await {
                        error!("Broadcasting error: {:?}", e);
                    }
                });

                app_handle.manage(broadcaster);

                // 启动监听
                let listener = match Listener::new(socket.clone(), app_state.clone()) {
                    Ok(l) => Arc::new(l),
                    Err(e) => {
                        error!("Failed to create listener: {:?}", e);
                        return;
                    }
                };

                let listener_clone: Arc<Listener> = listener.clone();
                tokio::spawn(async move {
                    if let Err(e) = listener_clone.start_listening().await {
                        error!("Listening error: {:?}", e);
                    }
                });
                app_handle.manage(listener);

                app_handle.manage(app_state);
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            #[cfg(desktop)]
            show_window,
            get_peers
        ])
        .run(tauri::generate_context!())
        .map_err(|e| {
            error!(error = ?e, "Failed to build app");
            e
        })
        .expect("Error while running tauri application");
}
