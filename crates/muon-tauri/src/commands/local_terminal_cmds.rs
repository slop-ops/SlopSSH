use base64::Engine;
use tauri::{Emitter, State};

use crate::AppState;

#[tauri::command]
pub async fn local_terminal_open(
    app: tauri::AppHandle,
    state: State<'_, tauri::async_runtime::Mutex<AppState>>,
    channel_id: String,
    cols: u16,
    rows: u16,
) -> Result<(), String> {
    tracing::debug!(channel_id = %channel_id, cols, rows, "local_terminal_open");
    let app_clone = app.clone();
    let cid = channel_id.clone();

    let on_data = Box::new(move |data: Vec<u8>| {
        let encoded = base64::engine::general_purpose::STANDARD.encode(&data);
        let _ = app_clone.emit(&format!("terminal-output-{}", cid), encoded);
    });

    let state = state.lock().await;
    let mut local = state.local_terminal.lock().map_err(|e| e.to_string())?;
    local
        .open(&channel_id, cols, rows, on_data)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn local_terminal_write(
    state: State<'_, tauri::async_runtime::Mutex<AppState>>,
    channel_id: String,
    data: String,
) -> Result<(), String> {
    tracing::debug!(channel_id = %channel_id, "local_terminal_write");
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(&data)
        .map_err(|e| e.to_string())?;
    let state = state.lock().await;
    let mut local = state.local_terminal.lock().map_err(|e| e.to_string())?;
    local
        .write(&channel_id, &decoded)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn local_terminal_resize(
    state: State<'_, tauri::async_runtime::Mutex<AppState>>,
    channel_id: String,
    cols: u16,
    rows: u16,
) -> Result<(), String> {
    tracing::debug!(channel_id = %channel_id, cols, rows, "local_terminal_resize");
    let state = state.lock().await;
    let local = state.local_terminal.lock().map_err(|e| e.to_string())?;
    local
        .resize(&channel_id, cols, rows)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn local_terminal_close(
    state: State<'_, tauri::async_runtime::Mutex<AppState>>,
    channel_id: String,
) -> Result<(), String> {
    tracing::debug!(channel_id = %channel_id, "local_terminal_close");
    let state = state.lock().await;
    let mut local = state.local_terminal.lock().map_err(|e| e.to_string())?;
    local.close(&channel_id);
    Ok(())
}
