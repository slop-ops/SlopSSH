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
