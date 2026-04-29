//! Recursive folder tree for organising saved sessions.

use serde::{Deserialize, Serialize};

use super::info::SessionInfo;

/// A folder that can contain sub-folders and session items.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SessionFolder {
    /// Unique folder identifier (UUID).
    pub id: String,
    /// User-visible folder name.
    pub name: String,
    /// Child folders.
    pub folders: Vec<SessionFolder>,
    /// Sessions stored directly in this folder.
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
    /// Creates a new folder with the given name and a fresh UUID.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::info::SessionInfo;

    #[test]
    fn test_default_folder() {
        let folder = SessionFolder::default();
        assert!(!folder.id.is_empty());
        assert!(folder.name.is_empty());
        assert!(folder.folders.is_empty());
        assert!(folder.items.is_empty());
    }

    #[test]
    fn test_new_folder() {
        let folder = SessionFolder::new("My Folder");
        assert!(!folder.id.is_empty());
        assert_eq!(folder.name, "My Folder");
        assert!(folder.folders.is_empty());
        assert!(folder.items.is_empty());
    }

    #[test]
    fn test_new_folder_unique_ids() {
        let f1 = SessionFolder::new("A");
        let f2 = SessionFolder::new("B");
        assert_ne!(f1.id, f2.id);
    }

    #[test]
    fn test_folder_with_nested_folders() {
        let mut parent = SessionFolder::new("Parent");
        let child1 = SessionFolder::new("Child 1");
        let child2 = SessionFolder::new("Child 2");
        parent.folders.push(child1);
        parent.folders.push(child2);
        assert_eq!(parent.folders.len(), 2);
        assert_eq!(parent.folders[0].name, "Child 1");
        assert_eq!(parent.folders[1].name, "Child 2");
    }

    #[test]
    fn test_folder_with_items() {
        let mut folder = SessionFolder::new("Sessions");
        let session = SessionInfo {
            name: "My Server".to_string(),
            host: "example.com".to_string(),
            ..Default::default()
        };
        folder.items.push(session);
        assert_eq!(folder.items.len(), 1);
        assert_eq!(folder.items[0].name, "My Server");
    }

    #[test]
    fn test_folder_serialize_deserialize() {
        let mut folder = SessionFolder::new("Test");
        folder.folders.push(SessionFolder::new("Sub"));
        let session = SessionInfo {
            name: "S1".to_string(),
            ..Default::default()
        };
        folder.items.push(session);

        let json = serde_json::to_string(&folder).unwrap();
        let parsed: SessionFolder = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "Test");
        assert_eq!(parsed.folders.len(), 1);
        assert_eq!(parsed.folders[0].name, "Sub");
        assert_eq!(parsed.items.len(), 1);
        assert_eq!(parsed.items[0].name, "S1");
    }

    #[test]
    fn test_folder_deep_nesting() {
        let mut root = SessionFolder::new("Root");
        let mut level1 = SessionFolder::new("L1");
        let mut level2 = SessionFolder::new("L2");
        level2.items.push(SessionInfo::default());
        level1.folders.push(level2);
        root.folders.push(level1);

        assert_eq!(root.folders[0].folders[0].items.len(), 1);
    }

    #[test]
    fn test_folder_default_serde_roundtrip() {
        let folder = SessionFolder::default();
        let json = serde_json::to_string(&folder).unwrap();
        let parsed: SessionFolder = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, folder.id);
        assert_eq!(parsed.name, folder.name);
    }

    #[test]
    fn test_folder_deserialize_missing_fields() {
        let json = r#"{"id":"fixed-id"}"#;
        let parsed: SessionFolder = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.id, "fixed-id");
        assert!(parsed.name.is_empty());
        assert!(parsed.folders.is_empty());
        assert!(parsed.items.is_empty());
    }

    #[test]
    fn test_folder_clone() {
        let mut folder = SessionFolder::new("Original");
        folder.items.push(SessionInfo::default());
        let cloned = folder.clone();
        assert_eq!(cloned.name, folder.name);
        assert_eq!(cloned.items.len(), folder.items.len());
        assert_eq!(cloned.id, folder.id);
    }
}
