use std::collections::HashMap;
use std::sync::Arc;

use russh_sftp::client::SftpSession;
use tokio::sync::Mutex;

use muon_core::config::settings::Settings;
use muon_core::credentials::CredentialCache;
use muon_core::file_transfer::engine::TransferEngine;
use muon_core::session::folder::SessionFolder;
use muon_core::session::pool::ConnectionPool;
use muon_core::session::store::SessionStore;
use muon_core::ssh::port_forward::PortForwardManager;
use muon_core::ssh::session_manager::SessionManager;

pub struct AppState {
    pub settings: Settings,
    pub session_store: SessionStore,
    #[allow(dead_code)]
    pub credential_cache: CredentialCache,
    pub ssh_manager: SessionManager,
    pub sftp_sessions: HashMap<String, Arc<Mutex<Option<SftpSession>>>>,
    pub transfer_engine: Arc<TransferEngine>,
    pub port_forward_manager: PortForwardManager,
    pub connection_pool: ConnectionPool,
}

impl AppState {
    pub fn new(settings: Settings, session_store: SessionStore) -> Self {
        Self {
            settings,
            session_store,
            credential_cache: CredentialCache::new(),
            ssh_manager: SessionManager::new(),
            sftp_sessions: HashMap::new(),
            transfer_engine: Arc::new(TransferEngine::new()),
            port_forward_manager: PortForwardManager::new(),
            connection_pool: ConnectionPool::new(3),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            settings: Settings::default(),
            session_store: SessionStore::from(SessionFolder::new("Root")),
            credential_cache: CredentialCache::new(),
            ssh_manager: SessionManager::new(),
            sftp_sessions: HashMap::new(),
            transfer_engine: Arc::new(TransferEngine::new()),
            port_forward_manager: PortForwardManager::new(),
            connection_pool: ConnectionPool::new(3),
        }
    }
}
