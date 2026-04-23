use base64::Engine;
use tauri::{Emitter, State};

use crate::AppState;

#[tauri::command]
pub async fn ssh_connect(
    state: State<'_, tauri::async_runtime::Mutex<AppState>>,
    session_id: String,
    password: Option<String>,
) -> Result<String, String> {
    let mut state = state.lock().await;

    let session_info = {
        let root = state.session_store.root();
        find_session(root, &session_id)
            .cloned()
            .ok_or_else(|| "Session not found".to_string())?
    };

    let auth = match password {
        Some(p) => muon_core::ssh::auth::AuthMethod::Password { password: p },
        None => {
            if let Some(ref key_path) = session_info.private_key_path {
                muon_core::ssh::auth::AuthMethod::PublicKey {
                    key_path: key_path.clone(),
                    passphrase: session_info.passphrase_key.clone(),
                }
            } else {
                return Err("No authentication method provided".to_string());
            }
        }
    };

    state
        .ssh_manager
        .connect(session_info, auth)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn ssh_disconnect(
    state: State<'_, tauri::async_runtime::Mutex<AppState>>,
    session_id: String,
) -> Result<(), String> {
    let mut state = state.lock().await;
    state
        .ssh_manager
        .disconnect(&session_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn ssh_open_shell(
    app: tauri::AppHandle,
    state: State<'_, tauri::async_runtime::Mutex<AppState>>,
    session_id: String,
    channel_id: String,
    cols: u16,
    rows: u16,
) -> Result<(), String> {
    let mut state = state.lock().await;
    state
        .ssh_manager
        .open_shell(&session_id, &channel_id, cols, rows)
        .await
        .map_err(|e| e.to_string())?;

    let app_clone = app.clone();
    let cid = channel_id.clone();
    state
        .ssh_manager
        .spawn_shell_read_loop(&session_id, &channel_id, move |data| {
            let encoded = base64::engine::general_purpose::STANDARD.encode(&data);
            let _ = app_clone.emit(&format!("terminal-output-{}", cid), encoded);
        })
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn ssh_write_shell(
    state: State<'_, tauri::async_runtime::Mutex<AppState>>,
    session_id: String,
    channel_id: String,
    data: String,
) -> Result<(), String> {
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(&data)
        .map_err(|e| e.to_string())?;
    let mut state = state.lock().await;
    state
        .ssh_manager
        .shell_write(&session_id, &channel_id, &decoded)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn ssh_resize_shell(
    state: State<'_, tauri::async_runtime::Mutex<AppState>>,
    session_id: String,
    channel_id: String,
    cols: u16,
    rows: u16,
) -> Result<(), String> {
    let state = state.lock().await;
    state
        .ssh_manager
        .shell_resize(&session_id, &channel_id, cols, rows)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn ssh_close_shell(
    state: State<'_, tauri::async_runtime::Mutex<AppState>>,
    session_id: String,
    channel_id: String,
) -> Result<(), String> {
    let mut state = state.lock().await;
    state
        .ssh_manager
        .close_shell(&session_id, &channel_id)
        .await
        .map_err(|e| e.to_string())
}

fn find_session<'a>(
    folder: &'a muon_core::session::folder::SessionFolder,
    id: &str,
) -> Option<&'a muon_core::session::info::SessionInfo> {
    for item in &folder.items {
        if item.id == id {
            return Some(item);
        }
    }
    for sub in &folder.folders {
        if let Some(found) = find_session(sub, id) {
            return Some(found);
        }
    }
    None
}
