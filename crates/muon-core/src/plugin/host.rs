use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::Mutex;

use super::api::{PluginCapability, PluginManifest};
use crate::config::paths;

pub struct PluginHost {
    allowed_capabilities: Vec<PluginCapability>,
    settings: HashMap<String, String>,
}

impl PluginHost {
    pub fn new(capabilities: &[PluginCapability]) -> Self {
        Self {
            allowed_capabilities: capabilities.to_vec(),
            settings: HashMap::new(),
        }
    }

    pub fn is_capability_allowed(&self, capability: &PluginCapability) -> bool {
        self.allowed_capabilities.contains(capability)
    }

    pub fn get_setting(&self, key: &str) -> Option<&str> {
        self.settings.get(key).map(|s| s.as_str())
    }

    pub fn set_setting(&mut self, key: &str, value: &str) {
        self.settings.insert(key.to_string(), value.to_string());
    }
}

pub struct LoadedPlugin {
    pub manifest: PluginManifest,
    pub wasm_path: PathBuf,
    pub enabled: bool,
}

pub struct PluginManager {
    plugins: Vec<LoadedPlugin>,
    hosts: HashMap<String, Arc<Mutex<PluginHost>>>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
            hosts: HashMap::new(),
        }
    }

    pub fn plugin_dir() -> anyhow::Result<PathBuf> {
        let dir = paths::config_dir()?.join("plugins");
        if !dir.exists() {
            std::fs::create_dir_all(&dir)?;
        }
        Ok(dir)
    }

    pub fn discover_plugins(&mut self) -> anyhow::Result<Vec<PluginManifest>> {
        let dir = Self::plugin_dir()?;
        let mut discovered = Vec::new();

        let entries = match std::fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => return Ok(discovered),
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "wasm") {
                let file_name = path
                    .file_stem()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");

                let manifest = PluginManifest {
                    id: file_name.to_string(),
                    name: file_name.to_string(),
                    version: "0.1.0".to_string(),
                    description: None,
                    author: None,
                    capabilities: vec![PluginCapability::ExecuteCommand],
                };

                if !self.plugins.iter().any(|p| p.manifest.id == manifest.id) {
                    self.plugins.push(LoadedPlugin {
                        manifest: manifest.clone(),
                        wasm_path: path.clone(),
                        enabled: true,
                    });
                    discovered.push(manifest);
                }
            }
        }

        Ok(discovered)
    }

    pub fn list_plugins(&self) -> Vec<&PluginManifest> {
        self.plugins.iter().map(|p| &p.manifest).collect()
    }

    pub fn set_enabled(&mut self, plugin_id: &str, enabled: bool) -> bool {
        if let Some(plugin) = self.plugins.iter_mut().find(|p| p.manifest.id == plugin_id) {
            plugin.enabled = enabled;
            true
        } else {
            false
        }
    }

    pub fn get_host(&self, plugin_id: &str) -> Option<Arc<Mutex<PluginHost>>> {
        self.hosts.get(plugin_id).cloned()
    }

    pub fn remove_plugin(&mut self, plugin_id: &str) -> bool {
        let before = self.plugins.len();
        self.plugins.retain(|p| p.manifest.id != plugin_id);
        self.hosts.remove(plugin_id);
        self.plugins.len() < before
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_host_new() {
        let host = PluginHost::new(&[PluginCapability::ExecuteCommand]);
        assert!(host.is_capability_allowed(&PluginCapability::ExecuteCommand));
        assert!(!host.is_capability_allowed(&PluginCapability::ReadSetting));
    }

    #[test]
    fn test_plugin_host_settings() {
        let mut host = PluginHost::new(&[]);
        assert!(host.get_setting("key").is_none());
        host.set_setting("key", "value");
        assert_eq!(host.get_setting("key"), Some("value"));
    }

    #[test]
    fn test_plugin_manager_new() {
        let mgr = PluginManager::new();
        assert!(mgr.list_plugins().is_empty());
    }

    #[test]
    fn test_plugin_manager_default() {
        let mgr = PluginManager::default();
        assert!(mgr.list_plugins().is_empty());
    }

    #[test]
    fn test_plugin_manager_set_enabled() {
        let mut mgr = PluginManager::new();
        mgr.plugins.push(LoadedPlugin {
            manifest: PluginManifest {
                id: "test".to_string(),
                name: "Test".to_string(),
                version: "1.0.0".to_string(),
                description: None,
                author: None,
                capabilities: vec![],
            },
            wasm_path: PathBuf::from("/tmp/test.wasm"),
            enabled: true,
        });

        assert!(mgr.set_enabled("test", false));
        assert!(!mgr.plugins[0].enabled);
        assert!(!mgr.set_enabled("nonexistent", false));
    }

    #[test]
    fn test_plugin_manager_remove() {
        let mut mgr = PluginManager::new();
        mgr.plugins.push(LoadedPlugin {
            manifest: PluginManifest {
                id: "test".to_string(),
                name: "Test".to_string(),
                version: "1.0.0".to_string(),
                description: None,
                author: None,
                capabilities: vec![],
            },
            wasm_path: PathBuf::from("/tmp/test.wasm"),
            enabled: true,
        });

        assert!(mgr.remove_plugin("test"));
        assert!(mgr.list_plugins().is_empty());
        assert!(!mgr.remove_plugin("nonexistent"));
    }

    #[test]
    fn test_plugin_manifest_serialize() {
        let manifest = PluginManifest {
            id: "test".to_string(),
            name: "Test Plugin".to_string(),
            version: "1.0.0".to_string(),
            description: Some("A test plugin".to_string()),
            author: Some("Author".to_string()),
            capabilities: vec![
                PluginCapability::ExecuteCommand,
                PluginCapability::ReadSetting,
            ],
        };
        let json = serde_json::to_string(&manifest).unwrap();
        assert!(json.contains("test"));
        assert!(json.contains("execute_command"));
    }

    #[test]
    fn test_plugin_panel_serialize() {
        let panel = super::super::api::PluginPanel {
            title: "Info".to_string(),
            content_type: super::super::api::PanelContentType::Markdown,
            content: "# Hello".to_string(),
        };
        let json = serde_json::to_string(&panel).unwrap();
        assert!(json.contains("markdown"));
    }
}
