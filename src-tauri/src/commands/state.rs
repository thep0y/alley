use std::sync::Arc;

use tauri::State;

use crate::{error::FluxyResult, state::app_state::AppState};

#[tauri::command]
pub async fn get_self_id(state: State<'_, Arc<AppState>>) -> FluxyResult<String> {
    Ok(state.get_self_id().to_owned())
}
