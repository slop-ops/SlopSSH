use muon_core::session::info::SessionInfo;
use tauri::State;

use crate::AppState;

#[tauri::command]
pub async fn list_sessions(
    state: State<'_, tauri::async_runtime::Mutex<AppState>>,
) -> Result<serde_json::Value, String> {
    let state = state.lock().await;
    serde_json::to_value(state.session_store.root()).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_session(
    state: State<'_, tauri::async_runtime::Mutex<AppState>>,
    session: serde_json::Value,
) -> Result<String, String> {
    let mut state = state.lock().await;
    let mut info: SessionInfo = serde_json::from_value(session).map_err(|e| e.to_string())?;
    if info.id.is_empty() {
        info.id = uuid::Uuid::new_v4().to_string();
    }
    let id = info.id.clone();
    state.session_store.add_session(None, info);
    state.session_store.save().map_err(|e| e.to_string())?;
    Ok(id)
}
