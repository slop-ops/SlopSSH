use muon_core::config::settings::Settings;
use muon_core::credentials::CredentialCache;
use muon_core::session::folder::SessionFolder;
use muon_core::session::store::SessionStore;

pub struct AppState {
    pub settings: Settings,
    pub session_store: SessionStore,
    #[allow(dead_code)]
    pub credential_cache: CredentialCache,
}

impl AppState {
    pub fn new(settings: Settings, session_store: SessionStore) -> Self {
        Self {
            settings,
            session_store,
            credential_cache: CredentialCache::new(),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            settings: Settings::default(),
            session_store: SessionStore::from(SessionFolder::new("Root")),
            credential_cache: CredentialCache::new(),
        }
    }
}
