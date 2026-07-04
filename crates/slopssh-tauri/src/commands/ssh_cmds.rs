use base64::Engine;
use tauri::{Emitter, State};

use crate::AppState;

#[tauri::command]
pub async fn ssh_connect(
    state: State<'_, AppState>,
    session_id: String,
    password: Option<String>,
    save_password: Option<bool>,
) -> Result<serde_json::Value, String> {
    tracing::info!(session_id = %session_id, "ssh_connect");

    let session_info = {
        let session_store = state.session_store.lock().await;
        let root = session_store.root();
        find_session(root, &session_id)
            .cloned()
            .ok_or_else(|| format!("Session '{}' not found in session store", session_id))?
    };

    session_info
        .validate()
        .map_err(|e| format!("Invalid session: {}", e))?;

    let auth = match session_info.auth_type {
        slopssh_core::session::AuthType::Password => {
            let p = if let Some(ref pw) = password {
                pw.clone()
            } else {
                let store = state.credential_store.lock().await;
                store
                    .get_credential(&session_id, "password")
                    .map_err(|e| e.to_string())?
                    .ok_or_else(|| "password_required".to_string())?
            };
            slopssh_core::ssh::auth::AuthMethod::Password { password: p }
        }
        slopssh_core::session::AuthType::PublicKey => {
            let key_path = session_info.private_key_path.clone().ok_or_else(|| {
                format!(
                    "No private key path configured for session '{}'",
                    session_id
                )
            })?;
            let passphrase = if let Some(ref pk) = session_info.passphrase_key {
                let store = state.credential_store.lock().await;
                store
                    .get_credential(pk, "passphrase")
                    .map_err(|e| e.to_string())?
            } else {
                let store = state.credential_store.lock().await;
                store
                    .get_credential(&session_id, "passphrase")
                    .map_err(|e| e.to_string())?
            };
            slopssh_core::ssh::auth::AuthMethod::PublicKey {
                key_path,
                passphrase,
            }
        }
        slopssh_core::session::AuthType::KeyboardInteractive => {
            let p = if let Some(ref pw) = password {
                pw.clone()
            } else {
                let store = state.credential_store.lock().await;
                store
                    .get_credential(&session_id, "password")
                    .map_err(|e| e.to_string())?
                    .ok_or_else(|| "password_required".to_string())?
            };
            slopssh_core::ssh::auth::AuthMethod::KeyboardInteractive { responses: vec![p] }
        }
        slopssh_core::session::AuthType::None => slopssh_core::ssh::auth::AuthMethod::None,
    };

    let enable_compression = {
        let settings = state.settings.lock().await;
        settings.enable_compression
    };

    let jump_credentials = resolve_jump_credentials(&state, &session_info).await;

    let mut ssh_manager = state.ssh_manager.lock().await;
    let result = ssh_manager
        .connect(session_info, auth, enable_compression, &jump_credentials)
        .await
        .map_err(|e| e.to_string())?;

    if save_password.unwrap_or(false) {
        if let Some(ref pw) = password {
            let store = state.credential_store.lock().await;
            if let Err(e) = store.save_credential(&session_id, "password", pw) {
                tracing::warn!(session_id = %session_id, error = %e, "Failed to save password to credential store");
            }
        }
    }

    serde_json::to_value(&result).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn ssh_is_connected(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<bool, String> {
    let ssh_manager = state.ssh_manager.lock().await;
    Ok(ssh_manager.get_handle(&session_id).is_some())
}

#[tauri::command]
pub async fn accept_host_key(state: State<'_, AppState>, session_id: String) -> Result<(), String> {
    tracing::info!(session_id = %session_id, "accept_host_key");
    let mut ssh_manager = state.ssh_manager.lock().await;
    ssh_manager
        .accept_host_key(&session_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn ssh_disconnect(state: State<'_, AppState>, session_id: String) -> Result<(), String> {
    tracing::info!(session_id = %session_id, "ssh_disconnect");
    {
        let mut sftp_sessions = state.sftp_sessions.lock().await;
        if let Some(sftp_arc) = sftp_sessions.remove(&session_id) {
            let mut guard = sftp_arc.lock().await;
            if let Some(sftp) = guard.take() {
                let _ = sftp.close().await;
            }
        }
    }
    {
        let mut pool = state.connection_pool.lock().await;
        pool.close_session(&session_id).await;
    }
    let mut ssh_manager = state.ssh_manager.lock().await;
    ssh_manager
        .disconnect(&session_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn ssh_open_shell(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    session_id: String,
    channel_id: String,
    cols: u16,
    rows: u16,
) -> Result<(), String> {
    tracing::debug!(session_id = %session_id, channel_id = %channel_id, cols, rows, "ssh_open_shell");
    let mut ssh_manager = state.ssh_manager.lock().await;
    ssh_manager
        .open_shell(&session_id, &channel_id, cols, rows)
        .await
        .map_err(|e| e.to_string())?;

    let app_clone = app.clone();
    let cid = channel_id.clone();
    ssh_manager
        .spawn_shell_read_loop(&session_id, &channel_id, move |data| {
            let encoded = base64::engine::general_purpose::STANDARD.encode(&data);
            if let Err(e) = app_clone.emit(&format!("terminal-output-{}", cid), encoded) {
                tracing::error!(channel_id = %cid, error = %e, "Failed to emit terminal output event");
            }
        })
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn ssh_write_shell(
    state: State<'_, AppState>,
    session_id: String,
    channel_id: String,
    data: String,
) -> Result<(), String> {
    tracing::debug!(session_id = %session_id, channel_id = %channel_id, "ssh_write_shell");
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(&data)
        .map_err(|e| e.to_string())?;
    let mut ssh_manager = state.ssh_manager.lock().await;
    ssh_manager
        .shell_write(&session_id, &channel_id, &decoded)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn ssh_resize_shell(
    state: State<'_, AppState>,
    session_id: String,
    channel_id: String,
    cols: u16,
    rows: u16,
) -> Result<(), String> {
    tracing::debug!(session_id = %session_id, channel_id = %channel_id, cols, rows, "ssh_resize_shell");
    let ssh_manager = state.ssh_manager.lock().await;
    ssh_manager
        .shell_resize(&session_id, &channel_id, cols, rows)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn ssh_close_shell(
    state: State<'_, AppState>,
    session_id: String,
    channel_id: String,
) -> Result<(), String> {
    tracing::debug!(session_id = %session_id, channel_id = %channel_id, "ssh_close_shell");
    let mut ssh_manager = state.ssh_manager.lock().await;
    ssh_manager
        .close_shell(&session_id, &channel_id)
        .await
        .map_err(|e| e.to_string())
}

fn find_session<'a>(
    folder: &'a slopssh_core::session::folder::SessionFolder,
    id: &str,
) -> Option<&'a slopssh_core::session::info::SessionInfo> {
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

async fn resolve_jump_credentials(
    state: &AppState,
    session_info: &slopssh_core::session::info::SessionInfo,
) -> std::collections::HashMap<String, String> {
    let mut creds = std::collections::HashMap::new();
    let store = state.credential_store.lock().await;

    for jh_str in &session_info.jump_hosts {
        if let Ok(jh) = serde_json::from_str::<slopssh_core::ssh::jump_host::JumpHost>(jh_str)
            && let Some(ref pk) = jh.password_key
            && let Ok(Some(password)) = store.get_credential(pk, "password")
        {
            creds.insert(pk.clone(), password);
        }
    }

    creds
}
