use tauri::State;

use crate::AppState;

#[tauri::command]
pub async fn plugin_list(
    state: State<'_, tauri::async_runtime::Mutex<AppState>>,
) -> Result<serde_json::Value, String> {
    let state = state.lock().await;
    let plugins = state.plugin_manager.list_plugins();
    serde_json::to_value(&plugins).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn plugin_discover(
    state: State<'_, tauri::async_runtime::Mutex<AppState>>,
) -> Result<serde_json::Value, String> {
    let mut state = state.lock().await;
    let discovered = state
        .plugin_manager
        .discover_plugins()
        .map_err(|e| e.to_string())?;
    serde_json::to_value(&discovered).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn plugin_set_enabled(
    state: State<'_, tauri::async_runtime::Mutex<AppState>>,
    plugin_id: String,
    enabled: bool,
) -> Result<(), String> {
    let mut state = state.lock().await;
    if state.plugin_manager.set_enabled(&plugin_id, enabled) {
        Ok(())
    } else {
        Err(format!("Plugin '{}' not found", plugin_id))
    }
}

#[tauri::command]
pub async fn plugin_remove(
    state: State<'_, tauri::async_runtime::Mutex<AppState>>,
    plugin_id: String,
) -> Result<(), String> {
    let mut state = state.lock().await;
    if state.plugin_manager.remove_plugin(&plugin_id) {
        Ok(())
    } else {
        Err(format!("Plugin '{}' not found", plugin_id))
    }
}
