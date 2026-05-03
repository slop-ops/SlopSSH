use tauri::State;

use crate::AppState;

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    tracing::debug!("get_settings");
    let settings = state.settings.lock().await;
    serde_json::to_value(&*settings).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_settings(
    state: State<'_, AppState>,
    settings: serde_json::Value,
) -> Result<(), String> {
    tracing::info!("save_settings");
    let mut settings_guard = state.settings.lock().await;
    let mut new_settings: slopssh_core::config::settings::Settings =
        serde_json::from_value(settings).map_err(|e| e.to_string())?;
    slopssh_core::config::settings::SettingsManager::save(&mut new_settings)
        .map_err(|e| e.to_string())?;
    *settings_guard = new_settings;
    Ok(())
}

#[tauri::command]
pub fn detect_editors() -> Result<serde_json::Value, String> {
    tracing::debug!("detect_editors");
    let editors = slopssh_core::config::editor::detect_editors();
    serde_json::to_value(&editors).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn open_in_editor(state: State<'_, AppState>, file_path: String) -> Result<(), String> {
    tracing::debug!(file_path = %file_path, "open_in_editor");
    let settings = state.settings.lock().await;
    let editor = &settings.external_editor;
    slopssh_core::config::editor::open_in_editor(editor, &file_path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_version() -> String {
    tracing::debug!("get_version");
    slopssh_core::version().to_string()
}

#[tauri::command]
pub async fn check_for_updates() -> Result<serde_json::Value, String> {
    tracing::info!("check_for_updates");
    let checker = slopssh_core::updater::github::UpdateChecker::new(
        "slop-ops",
        "SlopSSH",
        slopssh_core::version(),
    );
    match checker.check_for_update().await {
        Ok(Some(info)) => serde_json::to_value(&info).map_err(|e| e.to_string()),
        Ok(None) => Ok(serde_json::json!({"is_newer": false})),
        Err(e) => Err(format!("Update check failed: {}", e)),
    }
}

#[tauri::command]
pub async fn download_update(update_info: serde_json::Value) -> Result<serde_json::Value, String> {
    tracing::info!("download_update");
    let info: slopssh_core::updater::github::UpdateInfo =
        serde_json::from_value(update_info).map_err(|e| e.to_string())?;
    let checker = slopssh_core::updater::github::UpdateChecker::new(
        "slop-ops",
        "SlopSSH",
        slopssh_core::version(),
    );
    let path = checker
        .download_update(&info)
        .await
        .map_err(|e| format!("Download failed: {}", e))?;
    Ok(serde_json::json!({
        "path": path.to_string_lossy(),
        "size": std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0),
    }))
}

#[tauri::command]
pub async fn update_tray_tooltip(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let ssh_count = {
        let mgr = state.ssh_manager.lock().await;
        mgr.connected_session_ids().len()
    };
    let transfer_count = {
        let transfers = state.transfer_engine.list_progress().await;
        transfers
            .iter()
            .filter(|t| t.status == slopssh_core::file_transfer::progress::TransferStatus::InProgress)
            .count()
    };
    let mut parts = vec!["SlopSSH".to_string()];
    if ssh_count > 0 {
        parts.push(format!(
            "{} active session{}",
            ssh_count,
            if ssh_count > 1 { "s" } else { "" }
        ));
    }
    if transfer_count > 0 {
        parts.push(format!(
            "{} transfer{}",
            transfer_count,
            if transfer_count > 1 { "s" } else { "" }
        ));
    }
    let tooltip = parts.join(" | ");
    if let Some(tray) = app.tray_by_id("main") {
        let _ = tray.set_tooltip(Some(&tooltip));
    }
    Ok(())
}
