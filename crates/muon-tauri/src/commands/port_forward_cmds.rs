use tauri::State;

use crate::AppState;

#[tauri::command]
pub async fn port_forward_start(
    state: State<'_, AppState>,
    session_id: String,
    bind_host: String,
    bind_port: u16,
    target_host: String,
    target_port: u16,
    direction: String,
) -> Result<String, String> {
    tracing::info!(
        session_id = %session_id,
        bind_host = %bind_host,
        bind_port,
        target_host = %target_host,
        target_port,
        direction = %direction,
        "port_forward_start"
    );

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
        muon_core::ssh::port_forward::ForwardDirection::Local => {
            let handle = {
                let ssh_manager = state.ssh_manager.lock().await;
                ssh_manager
                    .get_handle(&session_id)
                    .ok_or_else(|| format!("No SSH connection for session '{}'", session_id))?
            };
            let mut pf = state.port_forward_manager.lock().await;
            pf.start_local(handle, rule).map_err(|e| e.to_string())?;
        }
        muon_core::ssh::port_forward::ForwardDirection::Remote => {
            let (handle, forward_map) = {
                let ssh_manager = state.ssh_manager.lock().await;
                let handle = ssh_manager
                    .get_handle(&session_id)
                    .ok_or_else(|| format!("No SSH connection for session '{}'", session_id))?;
                let forward_map = ssh_manager
                    .get_remote_forward_map(&session_id)
                    .ok_or_else(|| format!("No SSH connection for session '{}'", session_id))?;
                (handle, forward_map)
            };
            let mut pf = state.port_forward_manager.lock().await;
            pf.start_remote(handle, rule, forward_map)
                .await
                .map_err(|e| e.to_string())?;
        }
    };

    Ok(forward_id)
}

#[tauri::command]
pub async fn port_forward_stop(
    state: State<'_, AppState>,
    forward_id: String,
) -> Result<(), String> {
    tracing::info!(forward_id = %forward_id, "port_forward_stop");
    let mut pf = state.port_forward_manager.lock().await;
    pf.stop(&forward_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn port_forward_list(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    tracing::debug!("port_forward_list");
    let pf = state.port_forward_manager.lock().await;
    Ok(pf
        .list_active()
        .into_iter()
        .map(|s| s.to_string())
        .collect())
}
