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
    pub settings: Settings,
    pub session_store: SessionStore,
    #[allow(dead_code)]
    pub credential_cache: muon_core::credentials::CredentialCache,
    pub credential_store: CredentialStore,
    pub ssh_manager: SessionManager,
    pub sftp_sessions: HashMap<String, Arc<Mutex<Option<SftpSession>>>>,
    pub transfer_engine: Arc<TransferEngine>,
    pub port_forward_manager: PortForwardManager,
    pub connection_pool: ConnectionPool,
    pub local_terminal: std::sync::Mutex<LocalTerminalManager>,
    pub plugin_manager: PluginManager,
}

impl AppState {
    pub fn new(settings: Settings, session_store: SessionStore) -> Self {
        Self {
            settings,
            session_store,
            credential_cache: CredentialCache::new(),
            credential_store: CredentialStore::new_keyring_with_fallback(),
            ssh_manager: SessionManager::new(),
            sftp_sessions: HashMap::new(),
            transfer_engine: Arc::new(TransferEngine::new()),
            port_forward_manager: PortForwardManager::new(),
            connection_pool: ConnectionPool::new(3),
            local_terminal: std::sync::Mutex::new(LocalTerminalManager::new()),
            plugin_manager: PluginManager::new(),
        }
    }

    pub async fn shutdown(&mut self) {
        tracing::info!("App shutdown: cleaning up resources");

        self.port_forward_manager.stop_all().await;
        tracing::info!("Port forwards stopped");

        self.sftp_sessions.clear();
        tracing::info!("SFTP sessions dropped");

        self.connection_pool.close_all().await;
        tracing::info!("Connection pool closed");

        let session_ids = self.ssh_manager.connected_session_ids();
        for id in &session_ids {
            let _ = self.ssh_manager.disconnect(id).await;
        }
        if !session_ids.is_empty() {
            tracing::info!(count = session_ids.len(), "SSH sessions disconnected");
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
            settings: Settings::default(),
            session_store: SessionStore::from(SessionFolder::new("Root")),
            credential_cache: CredentialCache::new(),
            credential_store: CredentialStore::new_keyring_with_fallback(),
            ssh_manager: SessionManager::new(),
            sftp_sessions: HashMap::new(),
            transfer_engine: Arc::new(TransferEngine::new()),
            port_forward_manager: PortForwardManager::new(),
            connection_pool: ConnectionPool::new(3),
            local_terminal: std::sync::Mutex::new(LocalTerminalManager::new()),
            plugin_manager: PluginManager::new(),
        }
    }
}
