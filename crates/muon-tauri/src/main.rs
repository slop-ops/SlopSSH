use muon_core::config::settings::SettingsManager;
use muon_core::session::store::SessionStore;

mod commands;
mod state;

use commands::{
    create_folder, create_session, create_snippet, credential_delete, credential_get,
    credential_save, delete_session, delete_snippet, deploy_public_key, generate_key_pair,
    get_app_version, get_settings, greet, import_ssh_config, import_ssh_config_to_folder,
    list_local_keys, list_remote_keys, list_sessions, list_snippets, port_forward_list,
    port_forward_start, port_forward_stop, read_public_key, remote_exec, save_settings,
    sftp_connect, sftp_disconnect, sftp_home, sftp_list_dir, sftp_mkdir, sftp_read_file,
    sftp_remove, sftp_rename, sftp_stat, sftp_write_file, ssh_close_shell, ssh_connect,
    ssh_disconnect, ssh_open_shell, ssh_resize_shell, ssh_write_shell, transfer_cancel,
    transfer_clear_completed, transfer_download, transfer_list, transfer_upload, update_session,
    update_snippet,
};
use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let settings = SettingsManager::load().unwrap_or_default();
    muon_core::logging::init(&settings.log_level);

    let session_store = SessionStore::load().unwrap_or_else(|_| {
        let root = muon_core::session::folder::SessionFolder::new("Root");
        SessionStore::from(root)
    });

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
        ])
        .run(tauri::generate_context!())
        .expect("error while running Muon SSH");
}

fn main() {
    run();
}
