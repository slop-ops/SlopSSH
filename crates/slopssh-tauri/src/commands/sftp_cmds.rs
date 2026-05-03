use std::sync::Arc;
use std::time::UNIX_EPOCH;

use base64::Engine;
use russh_sftp::client::SftpSession;
use tauri::State;
use tokio::sync::Mutex;

use crate::AppState;

fn validate_path(path: &str) -> Result<String, String> {
    slopssh_core::utils::validate_sftp_path(path)
}

async fn get_sftp(
    state: &AppState,
    session_id: &str,
) -> Result<Arc<Mutex<Option<SftpSession>>>, String> {
    let sftp_sessions = state.sftp_sessions.lock().await;
    let entry = sftp_sessions
        .get(session_id)
        .ok_or_else(|| format!("No SFTP session for session '{}'", session_id))?;
    Ok(entry.clone())
}

#[tauri::command]
pub async fn sftp_connect(state: State<'_, AppState>, session_id: String) -> Result<(), String> {
    tracing::info!(session_id = %session_id, "SFTP connect");
    let sftp_session = {
        let ssh_manager = state.ssh_manager.lock().await;
        let channel = ssh_manager
            .open_sftp_channel(&session_id)
            .await
            .map_err(|e| {
                tracing::error!(session_id = %session_id, error = %e, "SFTP channel open failed");
                e.to_string()
            })?;
        let stream = channel.into_stream();
        let sftp = SftpSession::new(stream).await.map_err(|e| {
            tracing::error!(session_id = %session_id, error = %e, "SFTP session init failed");
            format!("SFTP init failed: {}", e)
        })?;
        Arc::new(Mutex::new(Some(sftp)))
    };

    let mut sftp_sessions = state.sftp_sessions.lock().await;
    sftp_sessions.insert(session_id, sftp_session);

    Ok(())
}

#[tauri::command]
pub async fn sftp_disconnect(state: State<'_, AppState>, session_id: String) -> Result<(), String> {
    tracing::info!(session_id = %session_id, "SFTP disconnect");
    let mut sftp_sessions = state.sftp_sessions.lock().await;
    if let Some(sftp_arc) = sftp_sessions.remove(&session_id) {
        let mut guard = sftp_arc.lock().await;
        if let Some(sftp) = guard.take()
            && let Err(e) = sftp.close().await
        {
            tracing::warn!(session_id = %session_id, error = %e, "SFTP close error");
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn sftp_list_dir(
    state: State<'_, AppState>,
    session_id: String,
    path: String,
) -> Result<serde_json::Value, String> {
    tracing::debug!(session_id = %session_id, path = %path, "SFTP list_dir");
    let path = validate_path(&path)?;
    let sftp_arc = get_sftp(&state, &session_id).await?;
    let guard = sftp_arc.lock().await;
    let sftp = guard
        .as_ref()
        .ok_or_else(|| format!("SFTP session closed for session '{}'", session_id))?;

    let read_dir = sftp.read_dir(&path).await.map_err(|e| e.to_string())?;

    let mut entries = Vec::new();
    for entry in read_dir {
        let name = entry.file_name();
        let meta = entry.metadata();
        let file_type = meta.file_type();

        let entry_path = if path == "/" {
            format!("/{}", name)
        } else {
            format!("{}/{}", path, name)
        };

        let modified = meta.modified().ok().and_then(|t| {
            t.duration_since(UNIX_EPOCH)
                .ok()
                .map(|d| d.as_millis() as u64)
        });

        entries.push(serde_json::json!({
            "name": name,
            "path": entry_path,
            "isDir": file_type.is_dir(),
            "isFile": file_type.is_file(),
            "isSymlink": file_type.is_symlink(),
            "size": meta.len(),
            "modified": modified,
        }));
    }

    Ok(serde_json::Value::Array(entries))
}

#[tauri::command]
pub async fn sftp_mkdir(
    state: State<'_, AppState>,
    session_id: String,
    path: String,
) -> Result<(), String> {
    tracing::debug!(session_id = %session_id, path = %path, "SFTP mkdir");
    let path = validate_path(&path)?;
    let sftp_arc = get_sftp(&state, &session_id).await?;
    let guard = sftp_arc.lock().await;
    let sftp = guard
        .as_ref()
        .ok_or_else(|| format!("SFTP session closed for session '{}'", session_id))?;
    sftp.create_dir(&path).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn sftp_remove(
    state: State<'_, AppState>,
    session_id: String,
    path: String,
) -> Result<(), String> {
    tracing::debug!(session_id = %session_id, path = %path, "SFTP remove");
    let path = validate_path(&path)?;
    let sftp_arc = get_sftp(&state, &session_id).await?;
    let guard = sftp_arc.lock().await;
    let sftp = guard
        .as_ref()
        .ok_or_else(|| format!("SFTP session closed for session '{}'", session_id))?;

    let meta = sftp.metadata(&path).await.map_err(|e| e.to_string())?;
    if meta.is_dir() {
        sftp.remove_dir(&path).await.map_err(|e| e.to_string())
    } else {
        sftp.remove_file(&path).await.map_err(|e| e.to_string())
    }
}

#[tauri::command]
pub async fn sftp_rename(
    state: State<'_, AppState>,
    session_id: String,
    from: String,
    to: String,
) -> Result<(), String> {
    tracing::debug!(session_id = %session_id, from = %from, to = %to, "SFTP rename");
    let from = validate_path(&from)?;
    let to = validate_path(&to)?;
    let sftp_arc = get_sftp(&state, &session_id).await?;
    let guard = sftp_arc.lock().await;
    let sftp = guard
        .as_ref()
        .ok_or_else(|| format!("SFTP session closed for session '{}'", session_id))?;
    sftp.rename(&from, &to).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn sftp_read_file(
    state: State<'_, AppState>,
    session_id: String,
    path: String,
) -> Result<String, String> {
    tracing::debug!(session_id = %session_id, path = %path, "SFTP read_file");
    let path = validate_path(&path)?;
    let sftp_arc = get_sftp(&state, &session_id).await?;
    let guard = sftp_arc.lock().await;
    let sftp = guard
        .as_ref()
        .ok_or_else(|| format!("SFTP session closed for session '{}'", session_id))?;
    let data = sftp.read(&path).await.map_err(|e| e.to_string())?;
    Ok(base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        &data,
    ))
}

#[tauri::command]
pub async fn sftp_write_file(
    state: State<'_, AppState>,
    session_id: String,
    path: String,
    data: String,
) -> Result<(), String> {
    tracing::debug!(session_id = %session_id, path = %path, "SFTP write_file");
    let path = validate_path(&path)?;
    let sftp_arc = get_sftp(&state, &session_id).await?;
    let guard = sftp_arc.lock().await;
    let sftp = guard
        .as_ref()
        .ok_or_else(|| format!("SFTP session closed for session '{}'", session_id))?;
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(&data)
        .map_err(|e| e.to_string())?;
    sftp.write(&path, &decoded).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn sftp_stat(
    state: State<'_, AppState>,
    session_id: String,
    path: String,
) -> Result<serde_json::Value, String> {
    tracing::debug!(session_id = %session_id, path = %path, "SFTP stat");
    let path = validate_path(&path)?;
    let sftp_arc = get_sftp(&state, &session_id).await?;
    let guard = sftp_arc.lock().await;
    let sftp = guard
        .as_ref()
        .ok_or_else(|| format!("SFTP session closed for session '{}'", session_id))?;

    let meta = sftp.metadata(&path).await.map_err(|e| e.to_string())?;
    let modified = meta.modified().ok().and_then(|t| {
        t.duration_since(UNIX_EPOCH)
            .ok()
            .map(|d| d.as_millis() as u64)
    });

    Ok(serde_json::json!({
        "isDir": meta.is_dir(),
        "isFile": meta.is_regular(),
        "isSymlink": meta.is_symlink(),
        "size": meta.len(),
        "modified": modified,
    }))
}

#[tauri::command]
pub async fn sftp_home(state: State<'_, AppState>, session_id: String) -> Result<String, String> {
    tracing::debug!(session_id = %session_id, "SFTP home");
    let sftp_arc = get_sftp(&state, &session_id).await?;
    let guard = sftp_arc.lock().await;
    let sftp = guard
        .as_ref()
        .ok_or_else(|| format!("SFTP session closed for session '{}'", session_id))?;
    sftp.canonicalize(".").await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn sftp_upload_sudo(
    state: State<'_, AppState>,
    session_id: String,
    remote_path: String,
    data: String,
) -> Result<(), String> {
    tracing::debug!(session_id = %session_id, path = %remote_path, "SFTP upload_sudo");
    let remote_path = validate_path(&remote_path)?;
    let sftp_arc = get_sftp(&state, &session_id).await?;
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(&data)
        .map_err(|e| e.to_string())?;

    let file_name = std::path::Path::new(&remote_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "slopssh_tmp".to_string());
    let tmp_path = format!("/tmp/.slopssh_upload_{}", file_name);

    {
        let guard = sftp_arc.lock().await;
        let sftp = guard
            .as_ref()
            .ok_or_else(|| format!("SFTP session closed for session '{}'", session_id))?;
        sftp.write(&tmp_path, &decoded)
            .await
            .map_err(|e| e.to_string())?;
    }

    let handle = {
        let ssh_manager = state.ssh_manager.lock().await;
        ssh_manager
            .get_handle(&session_id)
            .ok_or_else(|| format!("No SSH connection for session '{}'", session_id))?
    };

    let escaped_tmp = slopssh_core::utils::shell_escape(&tmp_path);
    let escaped_target = slopssh_core::utils::shell_escape(&remote_path);
    let cmd = format!(
        "sudo cp {} {} && rm -f {}",
        escaped_tmp, escaped_target, escaped_tmp
    );

    let result = slopssh_core::tools::remote_exec::RemoteExecutor::execute(&handle, &cmd, 60)
        .await
        .map_err(|e| e.to_string())?;

    if result.exit_code != 0 {
        let cleanup = format!("rm -f {}", slopssh_core::utils::shell_escape(&tmp_path));
        if let Err(e) =
            slopssh_core::tools::remote_exec::RemoteExecutor::execute(&handle, &cleanup, 10).await
        {
            tracing::warn!(session_id = %session_id, error = %e, "Sudo upload cleanup failed");
        }
        return Err(format!(
            "sudo cp failed (exit {}): {}",
            result.exit_code,
            result.stdout_string()
        ));
    }

    Ok(())
}

#[tauri::command]
pub async fn sftp_download_sudo(
    state: State<'_, AppState>,
    session_id: String,
    remote_path: String,
) -> Result<String, String> {
    tracing::debug!(session_id = %session_id, path = %remote_path, "SFTP download_sudo");
    let remote_path = validate_path(&remote_path)?;
    let handle = {
        let ssh_manager = state.ssh_manager.lock().await;
        ssh_manager
            .get_handle(&session_id)
            .ok_or_else(|| format!("No SSH connection for session '{}'", session_id))?
    };

    let file_name = std::path::Path::new(&remote_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "slopssh_tmp".to_string());
    let tmp_path = format!("/tmp/.slopssh_download_{}", file_name);

    let escaped_src = slopssh_core::utils::shell_escape(&remote_path);
    let escaped_tmp = slopssh_core::utils::shell_escape(&tmp_path);
    let cmd = format!("sudo cp {} {}", escaped_src, escaped_tmp);

    let result = slopssh_core::tools::remote_exec::RemoteExecutor::execute(&handle, &cmd, 60)
        .await
        .map_err(|e| e.to_string())?;

    if result.exit_code != 0 {
        return Err(format!(
            "sudo cp failed (exit {}): {}",
            result.exit_code,
            result.stdout_string()
        ));
    }

    let sftp_arc = get_sftp(&state, &session_id).await?;
    let data = {
        let guard = sftp_arc.lock().await;
        let sftp = guard
            .as_ref()
            .ok_or_else(|| format!("SFTP session closed for session '{}'", session_id))?;
        sftp.read(&tmp_path).await.map_err(|e| e.to_string())?
    };

    let cleanup = format!("rm -f {}", escaped_tmp);
    if let Err(e) =
        slopssh_core::tools::remote_exec::RemoteExecutor::execute(&handle, &cleanup, 10).await
    {
        tracing::warn!(session_id = %session_id, error = %e, "Sudo download cleanup failed");
    }

    Ok(base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        &data,
    ))
}
