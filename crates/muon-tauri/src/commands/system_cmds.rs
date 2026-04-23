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
