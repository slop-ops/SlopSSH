use muon_core::session::folder::SessionFolder;
use muon_core::session::info::SessionInfo;
use tauri::State;

use crate::AppState;

#[tauri::command]
pub async fn list_sessions(
    state: State<'_, tauri::async_runtime::Mutex<AppState>>,
) -> Result<serde_json::Value, String> {
    tracing::debug!("list_sessions");
    let state = state.lock().await;
    serde_json::to_value(state.session_store.root()).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_session(
    state: State<'_, tauri::async_runtime::Mutex<AppState>>,
    session: serde_json::Value,
) -> Result<String, String> {
    tracing::debug!("create_session");
    let mut state = state.lock().await;
    let mut info: SessionInfo = serde_json::from_value(session).map_err(|e| e.to_string())?;
    if info.id.is_empty() {
        info.id = uuid::Uuid::new_v4().to_string();
    }
    let id = info.id.clone();
    let folder_id = info.folder_id.clone();
    state.session_store.add_session(folder_id.as_deref(), info);
    state.session_store.save().map_err(|e| e.to_string())?;
    tracing::info!(session_id = %id, "Session created");
    Ok(id)
}

#[tauri::command]
pub async fn update_session(
    state: State<'_, tauri::async_runtime::Mutex<AppState>>,
    session: serde_json::Value,
) -> Result<(), String> {
    tracing::debug!("update_session");
    let mut state = state.lock().await;
    let updated: SessionInfo = serde_json::from_value(session).map_err(|e| e.to_string())?;
    let root = state.session_store.root_mut();
    remove_session_from_tree(root, &updated.id);
    let folder_id = updated.folder_id.clone();
    state
        .session_store
        .add_session(folder_id.as_deref(), updated);
    state.session_store.save().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_session(
    state: State<'_, tauri::async_runtime::Mutex<AppState>>,
    session_id: String,
) -> Result<(), String> {
    tracing::info!(session_id = %session_id, "delete_session");
    let mut state = state.lock().await;
    let root = state.session_store.root_mut();
    remove_session_from_tree(root, &session_id);
    state.session_store.save().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_folder(
    state: State<'_, tauri::async_runtime::Mutex<AppState>>,
    name: String,
    parent_id: Option<String>,
) -> Result<String, String> {
    tracing::debug!(name = %name, "create_folder");
    let mut state = state.lock().await;
    let folder = SessionFolder::new(&name);
    let id = folder.id.clone();
    state.session_store.add_folder(parent_id.as_deref(), folder);
    state.session_store.save().map_err(|e| e.to_string())?;
    Ok(id)
}

fn remove_session_from_tree(folder: &mut SessionFolder, id: &str) -> bool {
    let before = folder.items.len();
    folder.items.retain(|item| item.id != id);
    if folder.items.len() < before {
        return true;
    }
    for sub in &mut folder.folders {
        if remove_session_from_tree(sub, id) {
            return true;
        }
    }
    false
}
