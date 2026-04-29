use std::path::{Path, PathBuf};

use super::folder::SessionFolder;
use super::info::SessionInfo;
use crate::config::paths;

const MAX_BACKUPS: usize = 5;

pub struct SessionStore {
    root: SessionFolder,
    path: Option<PathBuf>,
}

impl SessionStore {
    pub fn load() -> anyhow::Result<Self> {
        let path = paths::sessions_file()?;
        Self::load_from(&path)
    }

    pub fn load_from(path: &Path) -> anyhow::Result<Self> {
        if !path.exists() {
            return Ok(Self {
                root: SessionFolder::new("Root"),
                path: Some(path.to_path_buf()),
            });
        }
        let content = std::fs::read_to_string(path)?;
        let root: SessionFolder = serde_json::from_str(&content)?;
        Ok(Self {
            root,
            path: Some(path.to_path_buf()),
        })
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let default_path = paths::sessions_file()?;
        let path = self.path.as_deref().unwrap_or(&default_path);
        self.save_to(path)
    }

    pub fn save_to(&self, path: &Path) -> anyhow::Result<()> {
        if let Some(parent) = path.parent()
            && !parent.exists()
        {
            std::fs::create_dir_all(parent)?;
        }
        if path.exists() {
            rotate_backups(path, MAX_BACKUPS)?;
        }
        let content = serde_json::to_string_pretty(&self.root)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn root(&self) -> &SessionFolder {
        &self.root
    }

    pub fn root_mut(&mut self) -> &mut SessionFolder {
        &mut self.root
    }

    pub fn add_session(&mut self, folder_id: Option<&str>, session: SessionInfo) {
        if let Some(fid) = folder_id
            && let Some(folder) = find_folder_mut(&mut self.root, fid)
        {
            folder.items.push(session);
            return;
        }
        self.root.items.push(session);
    }

    pub fn add_folder(&mut self, parent_id: Option<&str>, folder: SessionFolder) {
        if let Some(pid) = parent_id
            && let Some(parent) = find_folder_mut(&mut self.root, pid)
        {
            parent.folders.push(folder);
            return;
        }
        self.root.folders.push(folder);
    }
}

impl From<SessionFolder> for SessionStore {
    fn from(root: SessionFolder) -> Self {
        Self { root, path: None }
    }
}

fn rotate_backups(path: &Path, max_backups: usize) -> anyhow::Result<()> {
    let oldest = path.with_extension(format!("json.bak.{}", max_backups));
    if oldest.exists() {
        std::fs::remove_file(&oldest)?;
    }
    for i in (2..=max_backups).rev() {
        let src = path.with_extension(format!("json.bak.{}", i - 1));
        let dst = path.with_extension(format!("json.bak.{}", i));
        if src.exists() {
            std::fs::rename(&src, &dst)?;
        }
    }
    let first = path.with_extension("json.bak.1");
    std::fs::copy(path, &first)?;
    Ok(())
}

fn find_folder_mut<'a>(folder: &'a mut SessionFolder, id: &str) -> Option<&'a mut SessionFolder> {
    if folder.id == id {
        return Some(folder);
    }
    for sub in &mut folder.folders {
        if let Some(found) = find_folder_mut(sub, id) {
            return Some(found);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::AuthType;

    fn make_session(name: &str) -> SessionInfo {
        SessionInfo {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            host: format!("{}.example.com", name),
            port: 22,
            username: "root".to_string(),
            auth_type: AuthType::Password,
            ..Default::default()
        }
    }

    #[test]
    fn test_add_session_to_root() {
        let root = SessionFolder::new("Root");
        let mut store = SessionStore::from(root);
        let session = make_session("server1");
        store.add_session(None, session.clone());
        assert_eq!(store.root().items.len(), 1);
        assert_eq!(store.root().items[0].name, "server1");
    }

    #[test]
    fn test_add_session_to_folder() {
        let mut root = SessionFolder::new("Root");
        let folder = SessionFolder::new("Production");
        let folder_id = folder.id.clone();
        root.folders.push(folder);
        let mut store = SessionStore::from(root);
        let session = make_session("prod-server");
        store.add_session(Some(&folder_id), session);
        assert_eq!(store.root().items.len(), 0);
        assert_eq!(store.root().folders[0].items.len(), 1);
    }

    #[test]
    fn test_add_nested_folder() {
        let root = SessionFolder::new("Root");
        let mut store = SessionStore::from(root);
        let parent = SessionFolder::new("Parent");
        let parent_id = parent.id.clone();
        store.add_folder(None, parent);
        let child = SessionFolder::new("Child");
        store.add_folder(Some(&parent_id), child);
        assert_eq!(store.root().folders.len(), 1);
        assert_eq!(store.root().folders[0].folders.len(), 1);
        assert_eq!(store.root().folders[0].folders[0].name, "Child");
    }

    #[test]
    fn test_session_info_serialization() {
        let session = make_session("test");
        let json = serde_json::to_string(&session).unwrap();
        let parsed: SessionInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, session.name);
        assert_eq!(parsed.host, session.host);
        assert_eq!(parsed.port, session.port);
        assert_eq!(parsed.username, session.username);
    }

    #[test]
    fn test_folder_tree_serialization() {
        let mut root = SessionFolder::new("Root");
        let mut folder = SessionFolder::new("Servers");
        folder.items.push(make_session("s1"));
        folder.items.push(make_session("s2"));
        root.folders.push(folder);
        root.items.push(make_session("root-server"));

        let json = serde_json::to_string_pretty(&root).unwrap();
        let parsed: SessionFolder = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "Root");
        assert_eq!(parsed.folders.len(), 1);
        assert_eq!(parsed.folders[0].items.len(), 2);
        assert_eq!(parsed.items.len(), 1);
    }

    #[test]
    fn test_session_info_default() {
        let session = SessionInfo::default();
        assert!(!session.id.is_empty());
        assert_eq!(session.port, 22);
        assert_eq!(session.auth_type, AuthType::Password);
        assert_eq!(session.proxy_type, crate::session::ProxyType::None);
        assert!(!session.x11_forwarding);
        assert_eq!(session.encoding, "utf-8");
    }

    #[test]
    fn test_backup_rotation() {
        let dir = std::env::temp_dir().join("muon_test_backup_rotation");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("sessions.json");

        for i in 0..6 {
            let root = SessionFolder::new(&format!("Root {}", i));
            let store = SessionStore {
                root,
                path: Some(path.clone()),
            };
            store.save().unwrap();
        }

        assert!(path.exists());
        for i in 1..=5 {
            let bak = dir.join(format!("sessions.json.bak.{}", i));
            assert!(bak.exists(), "backup {} should exist", i);
        }
        let bak6 = dir.join("sessions.json.bak.6");
        assert!(!bak6.exists(), "backup 6 should not exist");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_backup_rotation_removes_oldest() {
        let dir = std::env::temp_dir().join("muon_test_backup_oldest");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("sessions.json");

        std::fs::write(&path, "marker_old").unwrap();
        let bak5 = dir.join("sessions.json.bak.5");
        std::fs::write(&bak5, "should_be_removed").unwrap();

        let root = SessionFolder::new("Root");
        let store = SessionStore {
            root,
            path: Some(path),
        };
        store.save().unwrap();

        assert!(!bak5.exists(), "oldest backup should be rotated away");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let dir = std::env::temp_dir().join("muon_test_roundtrip");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("sessions.json");

        let mut root = SessionFolder::new("Root");
        root.items.push(make_session("s1"));
        let store = SessionStore {
            root,
            path: Some(path.clone()),
        };
        store.save().unwrap();

        let loaded = SessionStore::load_from(&path).unwrap();
        assert_eq!(loaded.root().items.len(), 1);
        assert_eq!(loaded.root().items[0].name, "s1");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_backup_content_preserved() {
        let dir = std::env::temp_dir().join("muon_test_backup_content");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("sessions.json");

        let mut root1 = SessionFolder::new("V1");
        root1.items.push(make_session("old"));
        let store1 = SessionStore {
            root: root1,
            path: Some(path.clone()),
        };
        store1.save().unwrap();

        let mut root2 = SessionFolder::new("V2");
        root2.items.push(make_session("new"));
        let store2 = SessionStore {
            root: root2,
            path: Some(path.clone()),
        };
        store2.save().unwrap();

        let bak1 = dir.join("sessions.json.bak.1");
        let bak_content = std::fs::read_to_string(&bak1).unwrap();
        assert!(bak_content.contains("V1"));
        assert!(bak_content.contains("old"));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
