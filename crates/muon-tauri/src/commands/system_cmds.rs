use tauri::State;

use crate::AppState;

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! Welcome to Muon SSH.", name)
}

#[tauri::command]
pub fn get_settings(
    state: State<'_, tauri::async_runtime::Mutex<AppState>>,
) -> Result<serde_json::Value, String> {
    let state = state.blocking_lock();
    serde_json::to_value(&state.settings).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_settings(
    state: State<'_, tauri::async_runtime::Mutex<AppState>>,
    settings: serde_json::Value,
) -> Result<(), String> {
    let mut state = state.lock().await;
    let new_settings: muon_core::config::settings::Settings =
        serde_json::from_value(settings).map_err(|e| e.to_string())?;
    muon_core::config::settings::SettingsManager::save(&new_settings).map_err(|e| e.to_string())?;
    state.settings = new_settings;
    Ok(())
}

#[tauri::command]
pub fn detect_editors() -> Result<serde_json::Value, String> {
    let editors = muon_core::config::editor::detect_editors();
    serde_json::to_value(&editors).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn open_in_editor(
    state: State<'_, tauri::async_runtime::Mutex<AppState>>,
    file_path: String,
) -> Result<(), String> {
    let state = state.blocking_lock();
    let editor = &state.settings.external_editor;
    muon_core::config::editor::open_in_editor(editor, &file_path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_version() -> String {
    muon_core::version().to_string()
}

#[tauri::command]
pub async fn check_for_updates() -> Result<serde_json::Value, String> {
    let checker = muon_core::updater::github::UpdateChecker::new(
        "muon-ssh",
        "muon-ssh-rust",
        muon_core::version(),
    );
    match checker.check_for_update().await {
        Ok(Some(info)) => serde_json::to_value(&info).map_err(|e| e.to_string()),
        Ok(None) => Ok(serde_json::json!({"is_newer": false})),
        Err(e) => Err(format!("Update check failed: {}", e)),
    }
}
