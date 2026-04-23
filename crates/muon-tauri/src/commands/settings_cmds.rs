#[tauri::command]
pub async fn get_app_version() -> String {
    muon_core::version().to_string()
}
