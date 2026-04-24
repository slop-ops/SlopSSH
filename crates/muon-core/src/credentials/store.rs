use serde::{Deserialize, Serialize};

pub trait CredentialBackend: Send + Sync {
    fn save(&self, key: &str, value: &str) -> anyhow::Result<()>;
    fn get(&self, key: &str) -> anyhow::Result<Option<String>>;
    fn delete(&self, key: &str) -> anyhow::Result<()>;
}

pub struct KeyringBackend;

impl CredentialBackend for KeyringBackend {
    fn save(&self, key: &str, value: &str) -> anyhow::Result<()> {
        let entry = keyring::Entry::new("muon-ssh", key)?;
        entry.set_password(value)?;
        Ok(())
    }

    fn get(&self, key: &str) -> anyhow::Result<Option<String>> {
        let entry = keyring::Entry::new("muon-ssh", key)?;
        match entry.get_password() {
            Ok(val) => Ok(Some(val)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(anyhow::anyhow!("Keyring error: {}", e)),
        }
    }

    fn delete(&self, key: &str) -> anyhow::Result<()> {
        let entry = keyring::Entry::new("muon-ssh", key)?;
        match entry.delete_credential() {
            Ok(()) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(anyhow::anyhow!("Keyring error: {}", e)),
        }
    }
}

pub struct FileBackend;

#[derive(Serialize, Deserialize)]
struct CredentialEntry {
    key: String,
    value: String,
}

impl CredentialBackend for FileBackend {
    fn save(&self, key: &str, value: &str) -> anyhow::Result<()> {
        let path = credentials_file()?;
        let mut creds = load_all(&path)?;
        if let Some(existing) = creds.iter_mut().find(|c| c.key == key) {
            existing.value = value.to_string();
        } else {
            creds.push(CredentialEntry {
                key: key.to_string(),
                value: value.to_string(),
            });
        }
        let content = serde_json::to_string_pretty(&creds)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    fn get(&self, key: &str) -> anyhow::Result<Option<String>> {
        let path = credentials_file()?;
        let creds = load_all(&path)?;
        Ok(creds.into_iter().find(|c| c.key == key).map(|c| c.value))
    }

    fn delete(&self, key: &str) -> anyhow::Result<()> {
        let path = credentials_file()?;
        let mut creds = load_all(&path)?;
        let before = creds.len();
        creds.retain(|c| c.key != key);
        if creds.len() < before {
            let content = serde_json::to_string_pretty(&creds)?;
            std::fs::write(&path, content)?;
        }
        Ok(())
    }
}

fn credentials_file() -> anyhow::Result<std::path::PathBuf> {
    let dir = crate::config::paths::config_dir()?;
    Ok(dir.join("credentials.json"))
}

fn load_all(path: &std::path::Path) -> anyhow::Result<Vec<CredentialEntry>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = std::fs::read_to_string(path)?;
    let creds: Vec<CredentialEntry> = serde_json::from_str(&content)?;
    Ok(creds)
}

pub struct CredentialStore {
    backend: Box<dyn CredentialBackend>,
}

impl CredentialStore {
    pub fn new_keyring() -> Self {
        Self {
            backend: Box::new(KeyringBackend),
        }
    }

    pub fn new_file() -> Self {
        Self {
            backend: Box::new(FileBackend),
        }
    }

    pub fn new_keyring_with_fallback() -> Self {
        match keyring::Entry::new("muon-ssh", "__test__") {
            Ok(_) => {
                tracing::info!("Using OS keyring for credential storage");
                Self::new_keyring()
            }
            Err(_) => {
                tracing::warn!("OS keyring unavailable, falling back to file-based storage");
                Self::new_file()
            }
        }
    }

    pub fn key_for_session(session_id: &str, field: &str) -> String {
        format!("muon-ssh:{}:{}", session_id, field)
    }

    pub fn save_credential(&self, session_id: &str, field: &str, value: &str) -> anyhow::Result<()> {
        let key = Self::key_for_session(session_id, field);
        self.backend.save(&key, value)
    }

    pub fn get_credential(&self, session_id: &str, field: &str) -> anyhow::Result<Option<String>> {
        let key = Self::key_for_session(session_id, field);
        self.backend.get(&key)
    }

    pub fn delete_credential(&self, session_id: &str, field: &str) -> anyhow::Result<()> {
        let key = Self::key_for_session(session_id, field);
        self.backend.delete(&key)
    }

    pub fn delete_all_for_session(&self, session_id: &str) -> anyhow::Result<()> {
        for field in &["password", "passphrase", "proxy_password"] {
            let key = Self::key_for_session(session_id, field);
            let _ = self.backend.delete(&key);
        }
        Ok(())
    }
}

impl Default for CredentialStore {
    fn default() -> Self {
        Self::new_keyring_with_fallback()
    }
}
