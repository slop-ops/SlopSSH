use tauri::{Emitter, State};

use crate::AppState;

#[tauri::command]
pub async fn transfer_upload(
    app: tauri::AppHandle,
    state: State<'_, tauri::async_runtime::Mutex<AppState>>,
    transfer_id: String,
    session_id: String,
    local_path: String,
    remote_path: String,
    file_size: u64,
) -> Result<(), String> {
    tracing::debug!(transfer_id = %transfer_id, session_id = %session_id, file_size, "transfer_upload");
    let sftp = {
        let state = state.lock().await;
        state
            .sftp_sessions
            .get(&session_id)
            .ok_or_else(|| format!("No SFTP session established for session '{}'", session_id))?
            .clone()
    };

    let request = muon_core::file_transfer::progress::TransferRequest {
        id: transfer_id.clone(),
        session_id: session_id.clone(),
        direction: muon_core::file_transfer::progress::TransferDirection::Upload,
        source_path: local_path,
        dest_path: remote_path,
        file_size,
        conflict_resolution: muon_core::file_transfer::progress::ConflictResolution::Overwrite,
    };

    let engine = {
        let state = state.lock().await;
        state.transfer_engine.clone()
    };

    let app_clone = app.clone();
    let tid = transfer_id.clone();
    engine.spawn_upload(request, sftp, move |progress| {
        let _ = app_clone.emit(
            &format!("transfer-progress-{}", tid),
            serde_json::to_value(progress).unwrap_or_default(),
        );
    });

    Ok(())
}

#[tauri::command]
pub async fn transfer_download(
    app: tauri::AppHandle,
    state: State<'_, tauri::async_runtime::Mutex<AppState>>,
    transfer_id: String,
    session_id: String,
    remote_path: String,
    local_path: String,
    file_size: u64,
) -> Result<(), String> {
    tracing::debug!(transfer_id = %transfer_id, session_id = %session_id, file_size, "transfer_download");
    let sftp = {
        let state = state.lock().await;
        state
            .sftp_sessions
            .get(&session_id)
            .ok_or_else(|| format!("No SFTP session established for session '{}'", session_id))?
            .clone()
    };

    let request = muon_core::file_transfer::progress::TransferRequest {
        id: transfer_id.clone(),
        session_id: session_id.clone(),
        direction: muon_core::file_transfer::progress::TransferDirection::Download,
        source_path: remote_path,
        dest_path: local_path,
        file_size,
        conflict_resolution: muon_core::file_transfer::progress::ConflictResolution::Overwrite,
    };

    let engine = {
        let state = state.lock().await;
        state.transfer_engine.clone()
    };

    let app_clone = app.clone();
    let tid = transfer_id.clone();
    engine.spawn_download(request, sftp, move |progress| {
        let _ = app_clone.emit(
            &format!("transfer-progress-{}", tid),
            serde_json::to_value(progress).unwrap_or_default(),
        );
    });

    Ok(())
}

#[tauri::command]
pub async fn transfer_cancel(
    state: State<'_, tauri::async_runtime::Mutex<AppState>>,
    transfer_id: String,
) -> Result<bool, String> {
    tracing::debug!(transfer_id = %transfer_id, "transfer_cancel");
    let state = state.lock().await;
    state
        .transfer_engine
        .cancel(&transfer_id)
        .await
        .then_some(true)
        .ok_or_else(|| format!("Transfer '{}' not found or already completed", transfer_id))
}

#[tauri::command]
pub async fn transfer_list(
    state: State<'_, tauri::async_runtime::Mutex<AppState>>,
) -> Result<serde_json::Value, String> {
    tracing::debug!("transfer_list");
    let state = state.lock().await;
    let list = state.transfer_engine.list_progress().await;
    serde_json::to_value(&list).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn transfer_clear_completed(
    state: State<'_, tauri::async_runtime::Mutex<AppState>>,
) -> Result<(), String> {
    tracing::debug!("transfer_clear_completed");
    let state = state.lock().await;
    state.transfer_engine.clear_completed().await;
    Ok(())
}
