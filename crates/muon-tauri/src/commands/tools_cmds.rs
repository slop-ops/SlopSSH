use tauri::State;

use crate::AppState;

#[tauri::command]
pub async fn remote_exec(
    state: State<'_, tauri::async_runtime::Mutex<AppState>>,
    session_id: String,
    command: String,
    timeout_secs: Option<u64>,
) -> Result<serde_json::Value, String> {
    let state = state.lock().await;
    let handle = state
        .ssh_manager
        .get_handle(&session_id)
        .ok_or_else(|| "Not connected".to_string())?;

    let result = muon_core::tools::remote_exec::RemoteExecutor::execute(
        handle,
        &command,
        timeout_secs.unwrap_or(30),
    )
    .await
    .map_err(|e| e.to_string())?;

    Ok(serde_json::json!({
        "stdout": result.stdout_string(),
        "exitCode": result.exit_code,
    }))
}
