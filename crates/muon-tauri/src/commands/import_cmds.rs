use tauri::State;

use crate::AppState;

#[tauri::command]
pub async fn import_ssh_config(
    _state: State<'_, tauri::async_runtime::Mutex<AppState>>,
    path: Option<String>,
) -> Result<serde_json::Value, String> {
    let hosts = if let Some(ref p) = path {
        muon_core::session::import::SshConfigImporter::parse_file(std::path::Path::new(p))
            .map_err(|e| e.to_string())?
    } else {
        muon_core::session::import::SshConfigImporter::parse_default().map_err(|e| e.to_string())?
    };

    let sessions: Vec<serde_json::Value> = hosts
        .iter()
        .filter_map(|h| {
            muon_core::session::import::SshConfigImporter::to_session_info(h)
                .and_then(|info| serde_json::to_value(&info).ok())
        })
        .collect();

    Ok(serde_json::Value::Array(sessions))
}

#[tauri::command]
pub async fn import_ssh_config_to_folder(
    state: State<'_, tauri::async_runtime::Mutex<AppState>>,
    path: Option<String>,
) -> Result<String, String> {
    let hosts = if let Some(ref p) = path {
        muon_core::session::import::SshConfigImporter::parse_file(std::path::Path::new(p))
            .map_err(|e| e.to_string())?
    } else {
        muon_core::session::import::SshConfigImporter::parse_default().map_err(|e| e.to_string())?
    };

    let folder = muon_core::session::import::SshConfigImporter::import_to_folder(&hosts);
    let folder_id = folder.id.clone();

    let mut state = state.lock().await;
    state.session_store.add_folder(None, folder);
    state.session_store.save().map_err(|e| e.to_string())?;

    Ok(folder_id)
}

#[tauri::command]
pub async fn credential_save(
    state: State<'_, tauri::async_runtime::Mutex<AppState>>,
    session_id: String,
    field: String,
    value: String,
) -> Result<(), String> {
    let state = state.lock().await;
    state
        .credential_store
        .save_credential(&session_id, &field, &value)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn credential_get(
    state: State<'_, tauri::async_runtime::Mutex<AppState>>,
    session_id: String,
    field: String,
) -> Result<Option<String>, String> {
    let state = state.lock().await;
    state
        .credential_store
        .get_credential(&session_id, &field)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn credential_delete(
    state: State<'_, tauri::async_runtime::Mutex<AppState>>,
    session_id: String,
    field: String,
) -> Result<(), String> {
    let state = state.lock().await;
    state
        .credential_store
        .delete_credential(&session_id, &field)
        .map_err(|e| e.to_string())
}
