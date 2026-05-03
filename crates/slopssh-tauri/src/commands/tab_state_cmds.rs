#[tauri::command]
pub async fn save_tab_state(tabs: serde_json::Value) -> Result<(), String> {
    tracing::debug!("save_tab_state");
    let tab_state: slopssh_core::tab_state::TabState =
        serde_json::from_value(tabs).map_err(|e| e.to_string())?;
    tab_state.save().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn load_tab_state() -> Result<serde_json::Value, String> {
    tracing::debug!("load_tab_state");
    let state = slopssh_core::tab_state::TabState::load().map_err(|e| e.to_string())?;
    serde_json::to_value(&state).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn clear_tab_state() -> Result<(), String> {
    tracing::debug!("clear_tab_state");
    slopssh_core::tab_state::TabState::clear().map_err(|e| e.to_string())
}
