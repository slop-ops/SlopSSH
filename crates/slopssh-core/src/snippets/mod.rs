//! Reusable command snippets with JSON persistence.

use serde::{Deserialize, Serialize};

/// A saved command snippet that can be executed on remote sessions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snippet {
    /// Unique snippet identifier.
    pub id: String,
    /// User-visible snippet name.
    pub name: String,
    /// Shell command template.
    pub command: String,
    /// Optional human-readable description.
    pub description: Option<String>,
}

/// Load/save helper for snippet collections.
pub struct SnippetManager;

impl SnippetManager {
    /// Loads snippets from the configured `snippets.json` file.
    ///
    /// Returns an empty list when the file does not exist.
    pub fn load() -> anyhow::Result<Vec<Snippet>> {
        let path = crate::config::paths::snippets_file()?;
        if !path.exists() {
            return Ok(Vec::new());
        }
        let content = std::fs::read_to_string(&path)?;
        let snippets: Vec<Snippet> = serde_json::from_str(&content)?;
        Ok(snippets)
    }

    /// Persists a snippet collection to the configured `snippets.json` file.
    pub fn save(snippets: &[Snippet]) -> anyhow::Result<()> {
        let path = crate::config::paths::snippets_file()?;
        let content = serde_json::to_string_pretty(snippets)?;
        std::fs::write(&path, content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snippet_serialization() {
        let snippet = Snippet {
            id: "test-id".to_string(),
            name: "Check Disk".to_string(),
            command: "df -h".to_string(),
            description: Some("Show disk usage".to_string()),
        };
        let json = serde_json::to_string(&snippet).unwrap();
        let parsed: Snippet = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, snippet.id);
        assert_eq!(parsed.name, snippet.name);
        assert_eq!(parsed.command, snippet.command);
        assert_eq!(parsed.description, snippet.description);
    }

    #[test]
    fn test_snippet_optional_description() {
        let snippet = Snippet {
            id: "test-id".to_string(),
            name: "List Files".to_string(),
            command: "ls -la".to_string(),
            description: None,
        };
        let json = serde_json::to_string(&snippet).unwrap();
        let parsed: Snippet = serde_json::from_str(&json).unwrap();
        assert!(parsed.description.is_none());
    }

    #[test]
    fn test_snippets_roundtrip() {
        let snippets = vec![
            Snippet {
                id: "1".to_string(),
                name: "First".to_string(),
                command: "echo first".to_string(),
                description: None,
            },
            Snippet {
                id: "2".to_string(),
                name: "Second".to_string(),
                command: "echo second".to_string(),
                description: Some("desc".to_string()),
            },
        ];
        let json = serde_json::to_string_pretty(&snippets).unwrap();
        let parsed: Vec<Snippet> = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].name, "First");
        assert_eq!(parsed[1].command, "echo second");
    }
}
