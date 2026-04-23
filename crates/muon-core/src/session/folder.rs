use serde::{Deserialize, Serialize};

use super::info::SessionInfo;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SessionFolder {
    pub id: String,
    pub name: String,
    pub folders: Vec<SessionFolder>,
    pub items: Vec<SessionInfo>,
}

impl Default for SessionFolder {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: String::new(),
            folders: Vec::new(),
            items: Vec::new(),
        }
    }
}

impl SessionFolder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            ..Default::default()
        }
    }
}
