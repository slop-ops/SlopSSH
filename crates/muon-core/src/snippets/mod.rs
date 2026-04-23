use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snippet {
    pub id: String,
    pub name: String,
    pub command: String,
    pub description: Option<String>,
}

pub struct SnippetManager;

impl SnippetManager {
    pub fn load() -> anyhow::Result<Vec<Snippet>> {
        let path = crate::config::paths::snippets_file()?;
        if !path.exists() {
            return Ok(Vec::new());
        }
        let content = std::fs::read_to_string(&path)?;
        let snippets: Vec<Snippet> = serde_json::from_str(&content)?;
        Ok(snippets)
    }

    pub fn save(snippets: &[Snippet]) -> anyhow::Result<()> {
        let path = crate::config::paths::snippets_file()?;
        let content = serde_json::to_string_pretty(snippets)?;
        std::fs::write(&path, content)?;
        Ok(())
    }
}
