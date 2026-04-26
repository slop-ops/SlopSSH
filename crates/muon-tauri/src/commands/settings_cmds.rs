#[tauri::command]
pub async fn get_app_version() -> String {
    tracing::debug!("get_app_version");
    muon_core::version().to_string()
}
