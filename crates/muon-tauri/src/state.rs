use std::collections::HashMap;
use std::sync::Arc;

use russh_sftp::client::SftpSession;
use tokio::sync::Mutex;

use muon_core::config::settings::Settings;
use muon_core::credentials::CredentialCache;
use muon_core::credentials::store::CredentialStore;
use muon_core::file_transfer::engine::TransferEngine;
use muon_core::local_terminal::LocalTerminalManager;
use muon_core::plugin::host::PluginManager;
use muon_core::session::folder::SessionFolder;
use muon_core::session::pool::ConnectionPool;
use muon_core::session::store::SessionStore;
use muon_core::ssh::port_forward::PortForwardManager;
use muon_core::ssh::session_manager::SessionManager;

pub struct AppState {
    pub settings: Mutex<Settings>,
    pub session_store: Mutex<SessionStore>,
    pub ssh_manager: Mutex<SessionManager>,
    pub sftp_sessions: Mutex<HashMap<String, Arc<Mutex<Option<SftpSession>>>>>,
    pub transfer_engine: Arc<TransferEngine>,
    pub port_forward_manager: Mutex<PortForwardManager>,
    pub connection_pool: Mutex<ConnectionPool>,
    pub local_terminal: std::sync::Mutex<LocalTerminalManager>,
    pub plugin_manager: Mutex<PluginManager>,
    pub credential_store: Mutex<CredentialStore>,
    #[allow(dead_code)]
    pub credential_cache: CredentialCache,
}

impl AppState {
    pub fn new(settings: Settings, session_store: SessionStore) -> Self {
        Self {
            settings: Mutex::new(settings),
            session_store: Mutex::new(session_store),
            credential_cache: CredentialCache::new(),
            credential_store: Mutex::new(CredentialStore::new_keyring_with_fallback()),
            ssh_manager: Mutex::new(SessionManager::new()),
            sftp_sessions: Mutex::new(HashMap::new()),
            transfer_engine: Arc::new(TransferEngine::new()),
            port_forward_manager: Mutex::new(PortForwardManager::new()),
            connection_pool: Mutex::new(ConnectionPool::new(3)),
            local_terminal: std::sync::Mutex::new(LocalTerminalManager::new()),
            plugin_manager: Mutex::new(PluginManager::new()),
        }
    }

    pub async fn shutdown(&self) {
        tracing::info!("App shutdown: cleaning up resources");

        {
            let mut pf = self.port_forward_manager.lock().await;
            pf.stop_all().await;
        }
        tracing::info!("Port forwards stopped");

        {
            let mut sftp = self.sftp_sessions.lock().await;
            sftp.clear();
        }
        tracing::info!("SFTP sessions dropped");

        {
            let mut pool = self.connection_pool.lock().await;
            pool.close_all().await;
        }
        tracing::info!("Connection pool closed");

        {
            let mut mgr = self.ssh_manager.lock().await;
            let session_ids = mgr.connected_session_ids();
            for id in &session_ids {
                let _ = mgr.disconnect(id).await;
            }
            if !session_ids.is_empty() {
                tracing::info!(count = session_ids.len(), "SSH sessions disconnected");
            }
        }

        if let Ok(mut lt) = self.local_terminal.lock() {
            lt.close_all();
            tracing::info!("Local terminals closed");
        }

        self.transfer_engine.clear_completed().await;

        tracing::info!("App shutdown complete");
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            settings: Mutex::new(Settings::default()),
            session_store: Mutex::new(SessionStore::from(SessionFolder::new("Root"))),
            credential_cache: CredentialCache::new(),
            credential_store: Mutex::new(CredentialStore::new_keyring_with_fallback()),
            ssh_manager: Mutex::new(SessionManager::new()),
            sftp_sessions: Mutex::new(HashMap::new()),
            transfer_engine: Arc::new(TransferEngine::new()),
            port_forward_manager: Mutex::new(PortForwardManager::new()),
            connection_pool: Mutex::new(ConnectionPool::new(3)),
            local_terminal: std::sync::Mutex::new(LocalTerminalManager::new()),
            plugin_manager: Mutex::new(PluginManager::new()),
        }
    }
}
