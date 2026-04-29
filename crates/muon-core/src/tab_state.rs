use serde::{Deserialize, Serialize};

use crate::config::paths;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedTab {
    pub session_id: String,
    pub channel_id: String,
    pub title: String,
    pub is_local: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TabState {
    pub tabs: Vec<SavedTab>,
    pub active_tab_id: Option<String>,
}

impl TabState {
    pub fn load() -> anyhow::Result<Self> {
        let path = paths::tab_state_file()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(&path)?;
        let state: TabState = serde_json::from_str(&content)?;
        Ok(state)
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = paths::tab_state_file()?;
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    pub fn clear() -> anyhow::Result<()> {
        let path = paths::tab_state_file()?;
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_state_default() {
        let state = TabState::default();
        assert!(state.tabs.is_empty());
        assert!(state.active_tab_id.is_none());
    }

    #[test]
    fn test_tab_state_serialization() {
        let state = TabState {
            tabs: vec![SavedTab {
                session_id: "s1".to_string(),
                channel_id: "c1".to_string(),
                title: "server1".to_string(),
                is_local: false,
            }],
            active_tab_id: Some("c1".to_string()),
        };
        let json = serde_json::to_string(&state).unwrap();
        let parsed: TabState = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.tabs.len(), 1);
        assert_eq!(parsed.tabs[0].session_id, "s1");
        assert_eq!(parsed.active_tab_id, Some("c1".to_string()));
    }

    #[test]
    fn test_saved_tab_serialization() {
        let tab = SavedTab {
            session_id: "s2".to_string(),
            channel_id: "c2".to_string(),
            title: "local".to_string(),
            is_local: true,
        };
        let json = serde_json::to_string(&tab).unwrap();
        let parsed: SavedTab = serde_json::from_str(&json).unwrap();
        assert!(parsed.is_local);
        assert_eq!(parsed.title, "local");
    }
}
