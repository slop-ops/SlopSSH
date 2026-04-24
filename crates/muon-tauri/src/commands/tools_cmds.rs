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
    state: State<'_, tauri::async_runtime::Mutex<AppState>>,
    session_id: String,
    archive_path: String,
    sources: Vec<String>,
    format: String,
) -> Result<(), String> {
    let handle = {
        let state = state.lock().await;
        state
            .ssh_manager
            .get_handle(&session_id)
            .ok_or_else(|| "Not connected".to_string())?
    };

    let sources_str = sources
        .iter()
        .map(|s| shell_escape(s))
        .collect::<Vec<_>>()
        .join(" ");

    let command = match format.as_str() {
        "tar.gz" => format!("tar -czf {} {}", shell_escape(&archive_path), sources_str),
        "tar.bz2" => format!("tar -cjf {} {}", shell_escape(&archive_path), sources_str),
        "tar" => format!("tar -cf {} {}", shell_escape(&archive_path), sources_str),
        "zip" => format!("zip -r {} {}", shell_escape(&archive_path), sources_str),
        _ => return Err(format!("Unsupported archive format: {}", format)),
    };

    let result = muon_core::tools::remote_exec::RemoteExecutor::execute(
        &handle,
        &command,
        120,
    )
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
    state: State<'_, tauri::async_runtime::Mutex<AppState>>,
    session_id: String,
    archive_path: String,
    target_dir: String,
) -> Result<(), String> {
    let handle = {
        let state = state.lock().await;
        state
            .ssh_manager
            .get_handle(&session_id)
            .ok_or_else(|| "Not connected".to_string())?
    };

    let mkdir_cmd = format!("mkdir -p {}", shell_escape(&target_dir));
    let _ = muon_core::tools::remote_exec::RemoteExecutor::execute(&handle, &mkdir_cmd, 10)
        .await
        .map_err(|e| e.to_string())?;

    let command = if archive_path.ends_with(".tar.gz")
        || archive_path.ends_with(".tgz")
    {
        format!(
            "tar -xzf {} -C {}",
            shell_escape(&archive_path),
            shell_escape(&target_dir)
        )
    } else if archive_path.ends_with(".tar.bz2") {
        format!(
            "tar -xjf {} -C {}",
            shell_escape(&archive_path),
            shell_escape(&target_dir)
        )
    } else if archive_path.ends_with(".tar") {
        format!(
            "tar -xf {} -C {}",
            shell_escape(&archive_path),
            shell_escape(&target_dir)
        )
    } else if archive_path.ends_with(".zip") {
        format!(
            "unzip -o {} -d {}",
            shell_escape(&archive_path),
            shell_escape(&target_dir)
        )
    } else {
        format!(
            "tar -xf {} -C {} 2>/dev/null || unzip -o {} -d {}",
            shell_escape(&archive_path),
            shell_escape(&target_dir),
            shell_escape(&archive_path),
            shell_escape(&target_dir)
        )
    };

    let result = muon_core::tools::remote_exec::RemoteExecutor::execute(&handle, &command, 120)
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

fn shell_escape(s: &str) -> String {
    if s.contains(' ')
        || s.contains('"')
        || s.contains('\'')
        || s.contains('$')
        || s.contains('`')
        || s.contains('\\')
        || s.contains('(')
        || s.contains(')')
        || s.contains('&')
        || s.contains('|')
        || s.contains(';')
        || s.contains('<')
        || s.contains('>')
        || s.contains('*')
        || s.contains('?')
        || s.contains('~')
    {
        format!("'{}'", s.replace('\'', "'\\''"))
    } else {
        s.to_string()
    }
}
