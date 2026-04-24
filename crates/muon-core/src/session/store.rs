use super::folder::SessionFolder;
use super::info::SessionInfo;
use crate::config::paths;

pub struct SessionStore {
    root: SessionFolder,
}

impl SessionStore {
    pub fn load() -> anyhow::Result<Self> {
        let path = paths::sessions_file()?;
        if !path.exists() {
            return Ok(Self {
                root: SessionFolder::new("Root"),
            });
        }
        let content = std::fs::read_to_string(&path)?;
        let root: SessionFolder = serde_json::from_str(&content)?;
        Ok(Self { root })
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = paths::sessions_file()?;
        let content = serde_json::to_string_pretty(&self.root)?;
        std::fs::write(&path, content)?;
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
        Self { root }
    }
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
}
