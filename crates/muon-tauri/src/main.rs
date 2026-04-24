use muon_core::config::settings::SettingsManager;
use muon_core::session::store::SessionStore;
use tauri::{Emitter, Listener, Manager};

mod commands;
mod menu;
mod state;

use commands::{
    archive_create, archive_extract, check_for_updates, create_folder, create_session,
    create_snippet, credential_delete, credential_get, credential_save, delete_session,
    delete_snippet, deploy_public_key, detect_editors, generate_key_pair, get_app_version,
    get_settings, get_version, greet, import_ssh_config, import_ssh_config_to_folder,
    list_local_keys, list_remote_keys, list_sessions, list_snippets, local_terminal_close,
    local_terminal_open, local_terminal_resize, local_terminal_write, open_in_editor,
    plugin_discover, plugin_fire_event, plugin_get_all_settings, plugin_get_setting,
    plugin_list, plugin_remove, plugin_set_enabled, plugin_set_setting, plugin_show_notification,
    port_forward_list, port_forward_start, port_forward_stop, read_public_key, remote_exec,
    save_settings, sftp_connect, sftp_disconnect, sftp_download_sudo, sftp_home, sftp_list_dir,
    sftp_mkdir, sftp_read_file, sftp_remove, sftp_rename, sftp_stat, sftp_upload_sudo,
    sftp_write_file, ssh_close_shell, ssh_connect, ssh_disconnect, ssh_open_shell,
    ssh_resize_shell, ssh_write_shell, transfer_cancel, transfer_clear_completed,
    transfer_download, transfer_list, transfer_upload, update_session, update_snippet,
};
use state::AppState;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
struct WindowBounds {
    x: Option<i32>,
    y: Option<i32>,
    width: f64,
    height: f64,
}

fn bounds_file_path() -> Option<std::path::PathBuf> {
    muon_core::config::paths::config_dir()
        .ok()
        .map(|dir| dir.join("window_bounds.json"))
}

fn load_window_bounds() -> Option<WindowBounds> {
    let path = bounds_file_path()?;
    if !path.exists() {
        return None;
    }
    let content = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&content).ok()
}

fn save_window_bounds_on_close(window: &tauri::WebviewWindow) {
    let path = match bounds_file_path() {
        Some(p) => p,
        None => return,
    };

    let size = window.inner_size().ok();
    let pos = window.outer_position().ok();

    let bounds = WindowBounds {
        x: pos.map(|p| p.x),
        y: pos.map(|p| p.y),
        width: size.map(|s| s.width as f64).unwrap_or(1280.0),
        height: size.map(|s| s.height as f64).unwrap_or(800.0),
    };

    if let Ok(content) = serde_json::to_string_pretty(&bounds) {
        let _ = std::fs::write(&path, content);
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let settings = SettingsManager::load().unwrap_or_default();
    muon_core::logging::init(&settings.log_level);

    let session_store = SessionStore::load().unwrap_or_else(|_| {
        let root = muon_core::session::folder::SessionFolder::new("Root");
        SessionStore::from(root)
    });

    let saved_bounds = load_window_bounds();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(tauri::async_runtime::Mutex::new(AppState::new(
            settings,
            session_store,
        )))
        .invoke_handler(tauri::generate_handler![
            greet,
            get_settings,
            save_settings,
            list_sessions,
            create_session,
            update_session,
            delete_session,
            create_folder,
            get_app_version,
            ssh_connect,
            ssh_disconnect,
            ssh_open_shell,
            ssh_write_shell,
            ssh_resize_shell,
            ssh_close_shell,
            sftp_connect,
            sftp_disconnect,
            sftp_list_dir,
            sftp_mkdir,
            sftp_remove,
            sftp_rename,
            sftp_read_file,
            sftp_write_file,
            sftp_stat,
            sftp_home,
            list_snippets,
            create_snippet,
            update_snippet,
            delete_snippet,
            transfer_upload,
            transfer_download,
            transfer_cancel,
            transfer_list,
            transfer_clear_completed,
            remote_exec,
            archive_create,
            archive_extract,
            list_local_keys,
            list_remote_keys,
            generate_key_pair,
            deploy_public_key,
            read_public_key,
            port_forward_start,
            port_forward_stop,
            port_forward_list,
            import_ssh_config,
            import_ssh_config_to_folder,
            credential_save,
            credential_get,
            credential_delete,
            sftp_upload_sudo,
            sftp_download_sudo,
            local_terminal_open,
            local_terminal_write,
            local_terminal_resize,
            local_terminal_close,
            detect_editors,
            open_in_editor,
            get_version,
            check_for_updates,
            plugin_list,
            plugin_discover,
            plugin_set_enabled,
            plugin_remove,
            plugin_get_setting,
            plugin_set_setting,
            plugin_get_all_settings,
            plugin_fire_event,
            plugin_show_notification,
        ])
        .setup(move |app| {
            let app_menu = menu::create_menu(app.handle())?;
            app.set_menu(app_menu)?;

            menu::create_tray(app.handle())?;

            if let Some(saved) = saved_bounds
                && let Some(window) = app.get_webview_window("main")
            {
                let _ = window.set_size(tauri::LogicalSize::new(saved.width, saved.height));
                if let (Some(x), Some(y)) = (saved.x, saved.y) {
                    let _ = window.set_position(tauri::LogicalPosition::new(x, y));
                }
            }

            if let Some(window) = app.get_webview_window("main") {
                let window_clone = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        save_window_bounds_on_close(&window_clone);
                        api.prevent_close();
                        let _ = window_clone.hide();
                    }
                });
            }

            let app_handle = app.handle().clone();
            app.listen("tauri://file-drop", move |event: tauri::Event| {
                let payload = event.payload();
                if let Ok(files) = serde_json::from_str::<Vec<String>>(payload) {
                    for file in files {
                        if file.ends_with(".muon") {
                            let _ = app_handle.emit(
                                "open-session-file",
                                serde_json::json!({ "path": file }),
                            );
                        }
                    }
                }
            });

            Ok(())
        })
        .on_menu_event(|app, event| {
            let event_id = event.id.as_ref().to_string();
            let _ = app.emit("menu-event", &event_id);
        })
        .run(tauri::generate_context!())
        .expect("error while running Muon SSH");
}

fn main() {
    run();
}
