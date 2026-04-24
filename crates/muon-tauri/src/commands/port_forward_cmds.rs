use tauri::State;

use crate::AppState;

#[tauri::command]
pub async fn port_forward_start(
    state: State<'_, tauri::async_runtime::Mutex<AppState>>,
    session_id: String,
    bind_host: String,
    bind_port: u16,
    target_host: String,
    target_port: u16,
    direction: String,
) -> Result<String, String> {
    let mut state = state.lock().await;
    let handle = state
        .ssh_manager
        .get_handle(&session_id)
        .ok_or_else(|| format!("No SSH connection for session '{}'", session_id))?;

    let rule = match direction.as_str() {
        "local" => muon_core::ssh::port_forward::PortForwardRule::new_local(
            &bind_host,
            bind_port,
            &target_host,
            target_port,
        ),
        "remote" => muon_core::ssh::port_forward::PortForwardRule::new_remote(
            &bind_host,
            bind_port,
            &target_host,
            target_port,
        ),
        _ => return Err("Invalid direction: must be 'local' or 'remote'".to_string()),
    };

    let forward_id = rule.id.clone();

    match rule.direction {
        muon_core::ssh::port_forward::ForwardDirection::Local => state
            .port_forward_manager
            .start_local(handle, rule)
            .map_err(|e| e.to_string())?,
        muon_core::ssh::port_forward::ForwardDirection::Remote => {
            let forward_map = state
                .ssh_manager
                .get_remote_forward_map(&session_id)
                .ok_or_else(|| format!("No SSH connection for session '{}'", session_id))?;
            state
                .port_forward_manager
                .start_remote(handle, rule, forward_map)
                .await
                .map_err(|e| e.to_string())?
        }
    };

    Ok(forward_id)
}

#[tauri::command]
pub async fn port_forward_stop(
    state: State<'_, tauri::async_runtime::Mutex<AppState>>,
    forward_id: String,
) -> Result<(), String> {
    let mut state = state.lock().await;
    state
        .port_forward_manager
        .stop(&forward_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn port_forward_list(
    state: State<'_, tauri::async_runtime::Mutex<AppState>>,
) -> Result<Vec<String>, String> {
    let state = state.lock().await;
    Ok(state
        .port_forward_manager
        .list_active()
        .into_iter()
        .map(|s| s.to_string())
        .collect())
}
