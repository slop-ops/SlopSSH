use tauri::State;

use crate::AppState;

#[tauri::command]
pub async fn list_local_keys(
    _state: State<'_, tauri::async_runtime::Mutex<AppState>>,
) -> Result<Vec<serde_json::Value>, String> {
    let keys =
        muon_core::ssh::key_manager::KeyManager::list_local_keys().map_err(|e| e.to_string())?;

    Ok(keys
        .into_iter()
        .map(|k| {
            serde_json::json!({
                "path": k.path,
                "name": k.name,
                "keyType": k.key_type,
                "fingerprint": k.fingerprint,
                "hasPublicKey": k.has_public_key,
            })
        })
        .collect())
}

#[tauri::command]
pub async fn list_remote_keys(
    state: State<'_, tauri::async_runtime::Mutex<AppState>>,
    session_id: String,
) -> Result<Vec<serde_json::Value>, String> {
    let state = state.lock().await;
    let handle = state
        .ssh_manager
        .get_handle(&session_id)
        .ok_or_else(|| "Not connected".to_string())?;

    let keys = muon_core::ssh::key_manager::KeyManager::list_remote_keys(handle)
        .await
        .map_err(|e| e.to_string())?;

    Ok(keys
        .into_iter()
        .map(|k| {
            serde_json::json!({
                "path": k.path,
                "name": k.name,
                "keyType": k.key_type,
                "fingerprint": k.fingerprint,
                "hasPublicKey": k.has_public_key,
            })
        })
        .collect())
}

#[tauri::command]
pub async fn generate_key_pair(
    _state: State<'_, tauri::async_runtime::Mutex<AppState>>,
    algorithm: String,
    path: String,
    passphrase: Option<String>,
) -> Result<serde_json::Value, String> {
    let key = muon_core::ssh::key_manager::KeyManager::generate_key_pair(
        &algorithm,
        &path,
        passphrase.as_deref(),
    )
    .map_err(|e| e.to_string())?;

    Ok(serde_json::json!({
        "path": key.path,
        "name": key.name,
        "keyType": key.key_type,
        "fingerprint": key.fingerprint,
        "hasPublicKey": key.has_public_key,
    }))
}

#[tauri::command]
pub async fn deploy_public_key(
    state: State<'_, tauri::async_runtime::Mutex<AppState>>,
    session_id: String,
    public_key: String,
) -> Result<(), String> {
    let state = state.lock().await;
    let handle = state
        .ssh_manager
        .get_handle(&session_id)
        .ok_or_else(|| "Not connected".to_string())?;

    muon_core::ssh::key_manager::KeyManager::deploy_public_key(handle, &public_key)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn read_public_key(
    _state: State<'_, tauri::async_runtime::Mutex<AppState>>,
    path: String,
) -> Result<String, String> {
    muon_core::ssh::key_manager::KeyManager::read_public_key(&path).map_err(|e| e.to_string())
}
