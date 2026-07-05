use slopssh_core::session::folder::SessionFolder;
use slopssh_core::session::info::SessionInfo;
use tauri::State;

use crate::AppState;

#[tauri::command]
pub async fn list_sessions(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    tracing::debug!("list_sessions");
    let session_store = state.session_store.lock().await;
    serde_json::to_value(session_store.root()).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_session(
    state: State<'_, AppState>,
    session: serde_json::Value,
) -> Result<String, String> {
    tracing::debug!("create_session");
    let mut session_store = state.session_store.lock().await;
    let mut info: SessionInfo = serde_json::from_value(session).map_err(|e| e.to_string())?;
    if info.id.is_empty() {
        info.id = uuid::Uuid::new_v4().to_string();
    }
    info.validate()
        .map_err(|e| format!("Invalid session: {}", e))?;
    let id = info.id.clone();
    let folder_id = info.folder_id.clone();
    session_store.add_session(folder_id.as_deref(), info);
    session_store.save().map_err(|e| e.to_string())?;
    tracing::info!(session_id = %id, "Session created");
    Ok(id)
}

#[tauri::command]
pub async fn update_session(
    state: State<'_, AppState>,
    session: serde_json::Value,
) -> Result<(), String> {
    tracing::debug!("update_session");
    let mut session_store = state.session_store.lock().await;
    let updated: SessionInfo = serde_json::from_value(session).map_err(|e| e.to_string())?;
    updated
        .validate()
        .map_err(|e| format!("Invalid session: {}", e))?;
    let root = session_store.root_mut();
    remove_session_from_tree(root, &updated.id);
    let folder_id = updated.folder_id.clone();
    session_store.add_session(folder_id.as_deref(), updated);
    session_store.save().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_session(state: State<'_, AppState>, session_id: String) -> Result<(), String> {
    tracing::info!(session_id = %session_id, "delete_session");
    let mut session_store = state.session_store.lock().await;
    let root = session_store.root_mut();
    remove_session_from_tree(root, &session_id);
    session_store.save().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_folder(
    state: State<'_, AppState>,
    name: String,
    parent_id: Option<String>,
) -> Result<String, String> {
    tracing::debug!(name = %name, "create_folder");
    let mut session_store = state.session_store.lock().await;
    let folder = SessionFolder::new(&name);
    let id = folder.id.clone();
    session_store.add_folder(parent_id.as_deref(), folder);
    session_store.save().map_err(|e| e.to_string())?;
    Ok(id)
}

#[tauri::command]
pub async fn move_session(
    state: State<'_, AppState>,
    session_id: String,
    target_folder_id: Option<String>,
) -> Result<(), String> {
    tracing::debug!(session_id = %session_id, "move_session");
    let mut session_store = state.session_store.lock().await;
    let root = session_store.root_mut();

    let mut session: Option<SessionInfo> = None;
    extract_session_from_tree(root, &session_id, &mut session);

    let Some(session) = session else {
        return Err(format!("Session {} not found", session_id));
    };

    if let Some(fid) = target_folder_id.as_deref() {
        if let Some(folder) = find_folder_mut(root, fid) {
            folder.items.push(session);
        } else {
            root.items.push(session);
        }
    } else {
        root.items.push(session);
    }

    session_store.save().map_err(|e| e.to_string())
}

fn extract_session_from_tree(
    folder: &mut SessionFolder,
    id: &str,
    out: &mut Option<SessionInfo>,
) -> bool {
    let before = folder.items.len();
    folder.items.retain(|item| {
        if item.id == id {
            *out = Some(item.clone());
            false
        } else {
            true
        }
    });
    if folder.items.len() < before {
        return true;
    }
    for sub in &mut folder.folders {
        if extract_session_from_tree(sub, id, out) {
            return true;
        }
    }
    false
}

fn find_folder_mut<'a>(folder: &'a mut SessionFolder, id: &str) -> Option<&'a mut SessionFolder> {
    if folder.id == id {
        return Some(folder);
    }
    for sub in &mut folder.folders {
        if let Some(found) = find_folder_mut(sub, id) {
            return Some(found);
        }
    }
    None
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
