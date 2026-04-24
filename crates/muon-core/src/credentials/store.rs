use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredCredential {
    pub service: String,
    pub username: String,
    pub key: String,
}

pub struct CredentialStore;

impl CredentialStore {
    pub fn key_for_session(session_id: &str, field: &str) -> String {
        format!("muon-ssh:{}:{}", session_id, field)
    }

    pub fn save_credential(session_id: &str, field: &str, value: &str) -> anyhow::Result<()> {
        let path = Self::credentials_file()?;
        let key = Self::key_for_session(session_id, field);

        let mut creds = Self::load_all_raw()?;
        let entry = CredentialEntry {
            key,
            value: value.to_string(),
        };

        if let Some(existing) = creds.iter_mut().find(|c| c.key == entry.key) {
            existing.value = entry.value;
        } else {
            creds.push(entry);
        }

        let content = serde_json::to_string_pretty(&creds)?;
        std::fs::write(&path, content)?;

        Ok(())
    }

    pub fn get_credential(session_id: &str, field: &str) -> anyhow::Result<Option<String>> {
        let key = Self::key_for_session(session_id, field);
        let creds = Self::load_all_raw()?;
        Ok(creds.into_iter().find(|c| c.key == key).map(|c| c.value))
    }

    pub fn delete_credential(session_id: &str, field: &str) -> anyhow::Result<()> {
        let key = Self::key_for_session(session_id, field);
        let path = Self::credentials_file()?;

        let mut creds = Self::load_all_raw()?;
        let before = creds.len();
        creds.retain(|c| c.key != key);

        if creds.len() < before {
            let content = serde_json::to_string_pretty(&creds)?;
            std::fs::write(&path, content)?;
        }

        Ok(())
    }

    pub fn delete_all_for_session(session_id: &str) -> anyhow::Result<()> {
        let prefix = format!("muon-ssh:{}:", session_id);
        let path = Self::credentials_file()?;

        let mut creds = Self::load_all_raw()?;
        creds.retain(|c| !c.key.starts_with(&prefix));

        let content = serde_json::to_string_pretty(&creds)?;
        std::fs::write(&path, content)?;

        Ok(())
    }

    fn credentials_file() -> anyhow::Result<std::path::PathBuf> {
        let dir = crate::config::paths::config_dir()?;
        Ok(dir.join("credentials.json"))
    }

    fn load_all_raw() -> anyhow::Result<Vec<CredentialEntry>> {
        let path = Self::credentials_file()?;
        if !path.exists() {
            return Ok(Vec::new());
        }
        let content = std::fs::read_to_string(&path)?;
        let creds: Vec<CredentialEntry> = serde_json::from_str(&content)?;
        Ok(creds)
    }
}

#[derive(Serialize, Deserialize)]
struct CredentialEntry {
    key: String,
    value: String,
}
