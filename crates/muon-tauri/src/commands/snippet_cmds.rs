use muon_core::snippets::Snippet;

use crate::AppState;

#[tauri::command]
pub async fn list_snippets(
    _state: tauri::State<'_, tauri::async_runtime::Mutex<AppState>>,
) -> Result<serde_json::Value, String> {
    tracing::debug!("list_snippets");
    let snippets = muon_core::snippets::SnippetManager::load().map_err(|e| e.to_string())?;
    serde_json::to_value(&snippets).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_snippet(
    _state: tauri::State<'_, tauri::async_runtime::Mutex<AppState>>,
    snippet: serde_json::Value,
) -> Result<String, String> {
    tracing::debug!("create_snippet");
    let mut snippets = muon_core::snippets::SnippetManager::load().map_err(|e| e.to_string())?;
    let mut s: Snippet = serde_json::from_value(snippet).map_err(|e| e.to_string())?;
    if s.id.is_empty() {
        s.id = uuid::Uuid::new_v4().to_string();
    }
    let id = s.id.clone();
    snippets.push(s);
    muon_core::snippets::SnippetManager::save(&snippets).map_err(|e| e.to_string())?;
    Ok(id)
}

#[tauri::command]
pub async fn update_snippet(
    _state: tauri::State<'_, tauri::async_runtime::Mutex<AppState>>,
    snippet: serde_json::Value,
) -> Result<(), String> {
    tracing::debug!("update_snippet");
    let mut snippets = muon_core::snippets::SnippetManager::load().map_err(|e| e.to_string())?;
    let updated: Snippet = serde_json::from_value(snippet).map_err(|e| e.to_string())?;
    if let Some(idx) = snippets.iter().position(|s| s.id == updated.id) {
        snippets[idx] = updated;
    }
    muon_core::snippets::SnippetManager::save(&snippets).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_snippet(
    _state: tauri::State<'_, tauri::async_runtime::Mutex<AppState>>,
    snippet_id: String,
) -> Result<(), String> {
    tracing::debug!(snippet_id = %snippet_id, "delete_snippet");
    let mut snippets = muon_core::snippets::SnippetManager::load().map_err(|e| e.to_string())?;
    snippets.retain(|s| s.id != snippet_id);
    muon_core::snippets::SnippetManager::save(&snippets).map_err(|e| e.to_string())
}
