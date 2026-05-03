//! Persistent credential storage backends (OS keyring and encrypted file).

use serde::{Deserialize, Serialize};

/// Trait for credential persistence backends.
pub trait CredentialBackend: Send + Sync {
    /// Persists a key/value pair.
    fn save(&self, key: &str, value: &str) -> anyhow::Result<()>;
    /// Retrieves the value for a key, returning `None` if not found.
    fn get(&self, key: &str) -> anyhow::Result<Option<String>>;
    /// Deletes a credential entry. Succeeds even if the key does not exist.
    fn delete(&self, key: &str) -> anyhow::Result<()>;
}

pub struct KeyringBackend;

impl CredentialBackend for KeyringBackend {
    fn save(&self, key: &str, value: &str) -> anyhow::Result<()> {
        let entry = keyring_core::Entry::new("slopssh", key)?;
        entry.set_password(value)?;
        Ok(())
    }

    fn get(&self, key: &str) -> anyhow::Result<Option<String>> {
        let entry = keyring_core::Entry::new("slopssh", key)?;
        match entry.get_password() {
            Ok(val) => Ok(Some(val)),
            Err(keyring_core::Error::NoEntry) => Ok(None),
            Err(e) => Err(anyhow::anyhow!("Keyring error: {}", e)),
        }
    }

    fn delete(&self, key: &str) -> anyhow::Result<()> {
        let entry = keyring_core::Entry::new("slopssh", key)?;
        match entry.delete_credential() {
            Ok(()) => Ok(()),
            Err(keyring_core::Error::NoEntry) => Ok(()),
            Err(e) => Err(anyhow::anyhow!("Keyring error: {}", e)),
        }
    }
}

/// Credential backend backed by an AES-256-GCM encrypted JSON file.
pub struct FileBackend;

/// A single key/value entry in the file-based credential store.
#[derive(Serialize, Deserialize)]
struct CredentialEntry {
    key: String,
    value: String,
}

impl CredentialBackend for FileBackend {
    fn save(&self, key: &str, value: &str) -> anyhow::Result<()> {
        let path = credentials_file()?;
        let mut creds = load_all(&path)?;
        let encrypted = crate::utils::encrypt_value(value)?;
        if let Some(existing) = creds.iter_mut().find(|c| c.key == key) {
            existing.value = encrypted;
        } else {
            creds.push(CredentialEntry {
                key: key.to_string(),
                value: encrypted,
            });
        }
        let content = serde_json::to_string_pretty(&creds)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    fn get(&self, key: &str) -> anyhow::Result<Option<String>> {
        let path = credentials_file()?;
        let creds = load_all(&path)?;
        if let Some(entry) = creds.into_iter().find(|c| c.key == key) {
            match crate::utils::decrypt_value(&entry.value) {
                Ok(plaintext) => Ok(Some(plaintext)),
                Err(_) => Ok(Some(entry.value)),
            }
        } else {
            Ok(None)
        }
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

/// High-level credential store that delegates to a [`CredentialBackend`].
pub struct CredentialStore {
    backend: Box<dyn CredentialBackend>,
}

impl CredentialStore {
    /// Creates a store backed by the OS keyring.
    pub fn new_keyring() -> Self {
        Self {
            backend: Box::new(KeyringBackend),
        }
    }

    /// Creates a store backed by an encrypted file.
    pub fn new_file() -> Self {
        Self {
            backend: Box::new(FileBackend),
        }
    }

    /// Creates a store backed by the OS keyring, falling back to file-based
    /// storage if the keyring is unavailable.
    pub fn new_keyring_with_fallback() -> Self {
        if keyring::use_native_store(false).is_ok() {
            tracing::info!("Using OS keyring for credential storage");
            Self::new_keyring()
        } else {
            tracing::warn!(
                "OS keyring unavailable, falling back to encrypted file-based storage"
            );
            Self::new_file()
        }
    }

    /// Builds the composite key used to address a session credential field.
    pub fn key_for_session(session_id: &str, field: &str) -> String {
        format!("slopssh:{}:{}", session_id, field)
    }

    /// Persists a credential value for the given session and field.
    pub fn save_credential(
        &self,
        session_id: &str,
        field: &str,
        value: &str,
    ) -> anyhow::Result<()> {
        let key = Self::key_for_session(session_id, field);
        self.backend.save(&key, value)
    }

    /// Retrieves a credential value for the given session and field.
    pub fn get_credential(&self, session_id: &str, field: &str) -> anyhow::Result<Option<String>> {
        let key = Self::key_for_session(session_id, field);
        self.backend.get(&key)
    }

    /// Deletes a single credential for the given session and field.
    pub fn delete_credential(&self, session_id: &str, field: &str) -> anyhow::Result<()> {
        let key = Self::key_for_session(session_id, field);
        self.backend.delete(&key)
    }

    /// Removes all known credential fields (`password`, `passphrase`,
    /// `proxy_password`) for a session.
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    struct InMemoryBackend {
        store: Mutex<HashMap<String, String>>,
    }

    impl InMemoryBackend {
        fn new() -> Self {
            Self {
                store: Mutex::new(HashMap::new()),
            }
        }
    }

    impl CredentialBackend for InMemoryBackend {
        fn save(&self, key: &str, value: &str) -> anyhow::Result<()> {
            self.store
                .lock()
                .unwrap()
                .insert(key.to_string(), value.to_string());
            Ok(())
        }

        fn get(&self, key: &str) -> anyhow::Result<Option<String>> {
            Ok(self.store.lock().unwrap().get(key).cloned())
        }

        fn delete(&self, key: &str) -> anyhow::Result<()> {
            self.store.lock().unwrap().remove(key);
            Ok(())
        }
    }

    fn make_store(backend: InMemoryBackend) -> CredentialStore {
        CredentialStore {
            backend: Box::new(backend),
        }
    }

    #[test]
    fn test_key_for_session_format() {
        let key = CredentialStore::key_for_session("session-1", "password");
        assert_eq!(key, "slopssh:session-1:password");
    }

    #[test]
    fn test_key_for_session_different_fields() {
        let password_key = CredentialStore::key_for_session("s1", "password");
        let passphrase_key = CredentialStore::key_for_session("s1", "passphrase");
        let proxy_key = CredentialStore::key_for_session("s1", "proxy_password");
        assert_ne!(password_key, passphrase_key);
        assert_ne!(password_key, proxy_key);
        assert_ne!(passphrase_key, proxy_key);
    }

    #[test]
    fn test_key_for_session_different_sessions() {
        let k1 = CredentialStore::key_for_session("s1", "password");
        let k2 = CredentialStore::key_for_session("s2", "password");
        assert_ne!(k1, k2);
    }

    #[test]
    fn test_save_and_get_credential() {
        let store = make_store(InMemoryBackend::new());
        store.save_credential("s1", "password", "secret").unwrap();
        let val = store.get_credential("s1", "password").unwrap();
        assert_eq!(val, Some("secret".to_string()));
    }

    #[test]
    fn test_get_nonexistent_credential() {
        let store = make_store(InMemoryBackend::new());
        let val = store.get_credential("s1", "password").unwrap();
        assert!(val.is_none());
    }

    #[test]
    fn test_delete_credential() {
        let store = make_store(InMemoryBackend::new());
        store.save_credential("s1", "password", "secret").unwrap();
        store.delete_credential("s1", "password").unwrap();
        let val = store.get_credential("s1", "password").unwrap();
        assert!(val.is_none());
    }

    #[test]
    fn test_delete_nonexistent_credential() {
        let store = make_store(InMemoryBackend::new());
        assert!(store.delete_credential("s1", "password").is_ok());
    }

    #[test]
    fn test_save_overwrites_existing() {
        let store = make_store(InMemoryBackend::new());
        store.save_credential("s1", "password", "old").unwrap();
        store.save_credential("s1", "password", "new").unwrap();
        let val = store.get_credential("s1", "password").unwrap();
        assert_eq!(val, Some("new".to_string()));
    }

    #[test]
    fn test_delete_all_for_session() {
        let store = make_store(InMemoryBackend::new());
        store.save_credential("s1", "password", "p").unwrap();
        store.save_credential("s1", "passphrase", "pp").unwrap();
        store.save_credential("s1", "proxy_password", "px").unwrap();
        store.save_credential("s2", "password", "other").unwrap();

        store.delete_all_for_session("s1").unwrap();

        assert!(store.get_credential("s1", "password").unwrap().is_none());
        assert!(store.get_credential("s1", "passphrase").unwrap().is_none());
        assert!(
            store
                .get_credential("s1", "proxy_password")
                .unwrap()
                .is_none()
        );
        assert_eq!(
            store.get_credential("s2", "password").unwrap(),
            Some("other".to_string())
        );
    }

    #[test]
    fn test_load_all_empty_file() {
        let temp_dir = std::env::temp_dir().join("slopssh_test_creds_empty");
        let _ = std::fs::create_dir_all(&temp_dir);
        let path = temp_dir.join("creds.json");

        if path.exists() {
            std::fs::remove_file(&path).unwrap();
        }

        let result = load_all(&path);
        assert!(result.is_ok());
        let creds = result.unwrap();
        assert!(creds.is_empty());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_load_all_with_data() {
        let temp_dir = std::env::temp_dir().join("slopssh_test_creds_data");
        let _ = std::fs::create_dir_all(&temp_dir);
        let path = temp_dir.join("creds.json");

        let entries = vec![
            CredentialEntry {
                key: "k1".to_string(),
                value: "v1".to_string(),
            },
            CredentialEntry {
                key: "k2".to_string(),
                value: "v2".to_string(),
            },
        ];
        let content = serde_json::to_string_pretty(&entries).unwrap();
        std::fs::write(&path, &content).unwrap();

        let result = load_all(&path);
        assert!(result.is_ok());
        let creds = result.unwrap();
        assert_eq!(creds.len(), 2);
        assert_eq!(creds[0].key, "k1");
        assert_eq!(creds[1].value, "v2");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_credential_entry_serialize_deserialize() {
        let entry = CredentialEntry {
            key: "test-key".to_string(),
            value: "test-value".to_string(),
        };
        let json = serde_json::to_string(&entry).unwrap();
        let parsed: CredentialEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.key, "test-key");
        assert_eq!(parsed.value, "test-value");
    }
}
