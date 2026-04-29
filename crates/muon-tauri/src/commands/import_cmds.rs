use tauri::State;

use crate::AppState;

#[tauri::command]
pub async fn import_ssh_config(path: Option<String>) -> Result<serde_json::Value, String> {
    tracing::debug!(path = ?path, "import_ssh_config");
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
    state: State<'_, AppState>,
    path: Option<String>,
) -> Result<String, String> {
    tracing::debug!(path = ?path, "import_ssh_config_to_folder");
    let hosts = if let Some(ref p) = path {
        muon_core::session::import::SshConfigImporter::parse_file(std::path::Path::new(p))
            .map_err(|e| e.to_string())?
    } else {
        muon_core::session::import::SshConfigImporter::parse_default().map_err(|e| e.to_string())?
    };

    let folder = muon_core::session::import::SshConfigImporter::import_to_folder(&hosts);
    let folder_id = folder.id.clone();

    let mut session_store = state.session_store.lock().await;
    session_store.add_folder(None, folder);
    session_store.save().map_err(|e| e.to_string())?;

    Ok(folder_id)
}

#[tauri::command]
pub async fn credential_save(
    state: State<'_, AppState>,
    session_id: String,
    field: String,
    value: String,
) -> Result<(), String> {
    tracing::debug!(session_id = %session_id, field = %field, "credential_save");
    let credential_store = state.credential_store.lock().await;
    credential_store
        .save_credential(&session_id, &field, &value)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn credential_get(
    state: State<'_, AppState>,
    session_id: String,
    field: String,
) -> Result<Option<String>, String> {
    tracing::debug!(session_id = %session_id, field = %field, "credential_get");
    let credential_store = state.credential_store.lock().await;
    credential_store
        .get_credential(&session_id, &field)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn credential_delete(
    state: State<'_, AppState>,
    session_id: String,
    field: String,
) -> Result<(), String> {
    tracing::debug!(session_id = %session_id, field = %field, "credential_delete");
    let credential_store = state.credential_store.lock().await;
    credential_store
        .delete_credential(&session_id, &field)
        .map_err(|e| e.to_string())
}
