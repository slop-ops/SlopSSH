use std::collections::HashMap;

use tauri::{Emitter, State};

use crate::AppState;

#[tauri::command]
pub async fn plugin_list(
    state: State<'_, tauri::async_runtime::Mutex<AppState>>,
) -> Result<serde_json::Value, String> {
    tracing::debug!("plugin_list");
    let state = state.lock().await;
    let plugins: Vec<serde_json::Value> = state
        .plugin_manager
        .list_plugins_full()
        .iter()
        .map(|p| {
            serde_json::json!({
                "id": p.manifest.id,
                "name": p.manifest.name,
                "version": p.manifest.version,
                "description": p.manifest.description,
                "author": p.manifest.author,
                "capabilities": p.manifest.capabilities,
                "enabled": p.enabled,
            })
        })
        .collect();
    Ok(serde_json::Value::Array(plugins))
}

#[tauri::command]
pub async fn plugin_discover(
    state: State<'_, tauri::async_runtime::Mutex<AppState>>,
) -> Result<serde_json::Value, String> {
    tracing::info!("plugin_discover");
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
    tracing::info!(plugin_id = %plugin_id, enabled, "plugin_set_enabled");
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
    tracing::info!(plugin_id = %plugin_id, "plugin_remove");
    let mut state = state.lock().await;
    if state.plugin_manager.remove_plugin(&plugin_id) {
        Ok(())
    } else {
        Err(format!("Plugin '{}' not found", plugin_id))
    }
}

#[tauri::command]
pub async fn plugin_get_setting(
    state: State<'_, tauri::async_runtime::Mutex<AppState>>,
    plugin_id: String,
    key: String,
) -> Result<Option<String>, String> {
    tracing::debug!(plugin_id = %plugin_id, key = %key, "plugin_get_setting");
    let mut state = state.lock().await;
    Ok(state
        .plugin_manager
        .get_plugin_setting(&plugin_id, &key)
        .await)
}

#[tauri::command]
pub async fn plugin_set_setting(
    state: State<'_, tauri::async_runtime::Mutex<AppState>>,
    plugin_id: String,
    key: String,
    value: String,
) -> Result<(), String> {
    tracing::debug!(plugin_id = %plugin_id, key = %key, "plugin_set_setting");
    let mut state = state.lock().await;
    state
        .plugin_manager
        .set_plugin_setting(&plugin_id, &key, &value)
        .await;
    state
        .plugin_manager
        .save_settings_to_disk()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn plugin_get_all_settings(
    state: State<'_, tauri::async_runtime::Mutex<AppState>>,
    plugin_id: String,
) -> Result<HashMap<String, String>, String> {
    tracing::debug!(plugin_id = %plugin_id, "plugin_get_all_settings");
    let mut state = state.lock().await;
    Ok(state
        .plugin_manager
        .get_all_plugin_settings(&plugin_id)
        .await)
}

#[tauri::command]
pub async fn plugin_fire_event(
    app: tauri::AppHandle,
    state: State<'_, tauri::async_runtime::Mutex<AppState>>,
    plugin_id: String,
    event_type: String,
    payload: serde_json::Value,
) -> Result<(), String> {
    tracing::debug!(plugin_id = %plugin_id, event_type = %event_type, "plugin_fire_event");
    let state = state.lock().await;
    let event = muon_core::plugin::api::PluginEvent {
        event_type: event_type.clone(),
        payload: payload.clone(),
    };
    state.plugin_manager.fire_event(event);
    let _ = app.emit(
        &format!("plugin-event-{}", plugin_id),
        serde_json::json!({
            "pluginId": plugin_id,
            "eventType": event_type,
            "payload": payload,
        }),
    );
    Ok(())
}

#[tauri::command]
pub async fn plugin_show_notification(
    app: tauri::AppHandle,
    plugin_id: String,
    title: String,
    body: String,
) -> Result<(), String> {
    tracing::debug!(plugin_id = %plugin_id, title = %title, "plugin_show_notification");
    let _ = app.emit(
        "plugin-notification",
        serde_json::json!({
            "pluginId": plugin_id,
            "title": title,
            "body": body,
        }),
    );
    Ok(())
}
