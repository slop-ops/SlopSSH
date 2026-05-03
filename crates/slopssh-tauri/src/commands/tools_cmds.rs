use tauri::State;

use crate::AppState;

#[tauri::command]
pub async fn remote_exec(
    state: State<'_, AppState>,
    session_id: String,
    command: String,
    timeout_secs: Option<u64>,
) -> Result<serde_json::Value, String> {
    tracing::debug!(session_id = %session_id, "remote_exec");
    let ssh_manager = state.ssh_manager.lock().await;
    let handle = ssh_manager
        .get_handle(&session_id)
        .ok_or_else(|| format!("No SSH connection for session '{}'", session_id))?;

    let result = slopssh_core::tools::remote_exec::RemoteExecutor::execute(
        &handle,
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

#[tauri::command]
pub async fn archive_create(
    state: State<'_, AppState>,
    session_id: String,
    archive_path: String,
    sources: Vec<String>,
    format: String,
) -> Result<(), String> {
    tracing::debug!(session_id = %session_id, format = %format, "archive_create");
    let handle = {
        let ssh_manager = state.ssh_manager.lock().await;
        ssh_manager
            .get_handle(&session_id)
            .ok_or_else(|| "Not connected".to_string())?
    };

    let sources_str = sources
        .iter()
        .map(|s| slopssh_core::utils::shell_escape(s))
        .collect::<Vec<_>>()
        .join(" ");

    let command = match format.as_str() {
        "tar.gz" => format!(
            "tar -czf {} {}",
            slopssh_core::utils::shell_escape(&archive_path),
            sources_str
        ),
        "tar.bz2" => format!(
            "tar -cjf {} {}",
            slopssh_core::utils::shell_escape(&archive_path),
            sources_str
        ),
        "tar" => format!(
            "tar -cf {} {}",
            slopssh_core::utils::shell_escape(&archive_path),
            sources_str
        ),
        "zip" => format!(
            "zip -r {} {}",
            slopssh_core::utils::shell_escape(&archive_path),
            sources_str
        ),
        _ => return Err(format!("Unsupported archive format: {}", format)),
    };

    let result = slopssh_core::tools::remote_exec::RemoteExecutor::execute(&handle, &command, 120)
        .await
        .map_err(|e| e.to_string())?;

    if result.exit_code != 0 {
        return Err(format!(
            "Archive creation failed (exit {}): {}",
            result.exit_code,
            result.stdout_string()
        ));
    }

    Ok(())
}

#[tauri::command]
pub async fn archive_extract(
    state: State<'_, AppState>,
    session_id: String,
    archive_path: String,
    target_dir: String,
) -> Result<(), String> {
    tracing::debug!(session_id = %session_id, "archive_extract");
    let handle = {
        let ssh_manager = state.ssh_manager.lock().await;
        ssh_manager
            .get_handle(&session_id)
            .ok_or_else(|| "Not connected".to_string())?
    };

    let mkdir_cmd = format!("mkdir -p {}", slopssh_core::utils::shell_escape(&target_dir));
    let _ = slopssh_core::tools::remote_exec::RemoteExecutor::execute(&handle, &mkdir_cmd, 10)
        .await
        .map_err(|e| e.to_string())?;

    let command = if archive_path.ends_with(".tar.gz") || archive_path.ends_with(".tgz") {
        format!(
            "tar -xzf {} -C {}",
            slopssh_core::utils::shell_escape(&archive_path),
            slopssh_core::utils::shell_escape(&target_dir)
        )
    } else if archive_path.ends_with(".tar.bz2") {
        format!(
            "tar -xjf {} -C {}",
            slopssh_core::utils::shell_escape(&archive_path),
            slopssh_core::utils::shell_escape(&target_dir)
        )
    } else if archive_path.ends_with(".tar") {
        format!(
            "tar -xf {} -C {}",
            slopssh_core::utils::shell_escape(&archive_path),
            slopssh_core::utils::shell_escape(&target_dir)
        )
    } else if archive_path.ends_with(".zip") {
        format!(
            "unzip -o {} -d {}",
            slopssh_core::utils::shell_escape(&archive_path),
            slopssh_core::utils::shell_escape(&target_dir)
        )
    } else {
        format!(
            "tar -xf {} -C {} 2>/dev/null || unzip -o {} -d {}",
            slopssh_core::utils::shell_escape(&archive_path),
            slopssh_core::utils::shell_escape(&target_dir),
            slopssh_core::utils::shell_escape(&archive_path),
            slopssh_core::utils::shell_escape(&target_dir)
        )
    };

    let result = slopssh_core::tools::remote_exec::RemoteExecutor::execute(&handle, &command, 120)
        .await
        .map_err(|e| e.to_string())?;

    if result.exit_code != 0 {
        return Err(format!(
            "Archive extraction failed (exit {}): {}",
            result.exit_code,
            result.stdout_string()
        ));
    }

    Ok(())
}
