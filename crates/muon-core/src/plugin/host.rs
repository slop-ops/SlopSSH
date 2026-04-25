use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::Mutex;

use super::api::{PluginCapability, PluginEvent, PluginManifest, PluginPanel};
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

    pub fn all_settings(&self) -> &HashMap<String, String> {
        &self.settings
    }
}

pub struct LoadedPlugin {
    pub manifest: PluginManifest,
    pub wasm_path: PathBuf,
    pub enabled: bool,
}

type PluginEventCallback = Box<dyn Fn(PluginEvent) + Send + Sync>;

#[derive(Default)]
pub struct PluginManager {
    plugins: Vec<LoadedPlugin>,
    hosts: HashMap<String, Arc<Mutex<PluginHost>>>,
    event_callbacks: Vec<PluginEventCallback>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
            hosts: HashMap::new(),
            event_callbacks: Vec::new(),
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

    pub fn get_plugin(&self, plugin_id: &str) -> Option<&LoadedPlugin> {
        self.plugins.iter().find(|p| p.manifest.id == plugin_id)
    }

    pub fn list_plugins_full(&self) -> Vec<&LoadedPlugin> {
        self.plugins.iter().collect()
    }

    pub fn ensure_host(&mut self, plugin_id: &str) -> Arc<Mutex<PluginHost>> {
        if !self.hosts.contains_key(plugin_id) {
            let plugin = self.plugins.iter().find(|p| p.manifest.id == plugin_id);
            let caps = plugin
                .map(|p| p.manifest.capabilities.as_slice())
                .unwrap_or(&[]);
            self.hosts.insert(
                plugin_id.to_string(),
                Arc::new(Mutex::new(PluginHost::new(caps))),
            );
        }
        self.hosts.get(plugin_id).cloned().unwrap()
    }

    pub async fn get_plugin_setting(&mut self, plugin_id: &str, key: &str) -> Option<String> {
        let host = self.ensure_host(plugin_id);
        let host = host.lock().await;
        host.get_setting(key).map(|s| s.to_string())
    }

    pub async fn set_plugin_setting(&mut self, plugin_id: &str, key: &str, value: &str) {
        let host = self.ensure_host(plugin_id);
        let mut host = host.lock().await;
        host.set_setting(key, value);
    }

    pub async fn get_all_plugin_settings(&mut self, plugin_id: &str) -> HashMap<String, String> {
        let host = self.ensure_host(plugin_id);
        let host = host.lock().await;
        host.all_settings().clone()
    }

    pub fn on_event<F>(&mut self, callback: F)
    where
        F: Fn(PluginEvent) + Send + Sync + 'static,
    {
        self.event_callbacks.push(Box::new(callback));
    }

    pub fn fire_event(&self, event: PluginEvent) {
        for cb in &self.event_callbacks {
            cb(event.clone());
        }
    }

    pub fn load_settings_from_disk(&mut self) -> anyhow::Result<()> {
        let dir = Self::plugin_dir()?;
        let settings_file = dir.join("plugin_settings.json");
        if !settings_file.exists() {
            return Ok(());
        }
        let content = std::fs::read_to_string(&settings_file)?;
        let all_settings: HashMap<String, HashMap<String, String>> =
            serde_json::from_str(&content)?;
        for (plugin_id, settings) in all_settings {
            let host = self.ensure_host(&plugin_id);
            let host_arc = host.clone();
            let settings = settings.clone();
            std::thread::spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build();
                if let Ok(rt) = rt {
                    rt.block_on(async {
                        let mut h = host_arc.lock().await;
                        for (k, v) in settings {
                            h.set_setting(&k, &v);
                        }
                    });
                }
            });
        }
        Ok(())
    }

    pub fn save_settings_to_disk(&self) -> anyhow::Result<()> {
        let dir = Self::plugin_dir()?;
        let mut all_settings: HashMap<String, HashMap<String, String>> = HashMap::new();
        let hosts = &self.hosts;
        for plugin in &self.plugins {
            if let Some(host_arc) = hosts.get(&plugin.manifest.id)
                && let Ok(host) = host_arc.try_lock()
            {
                all_settings.insert(plugin.manifest.id.clone(), host.all_settings().clone());
            }
        }
        let content = serde_json::to_string_pretty(&all_settings)?;
        std::fs::write(dir.join("plugin_settings.json"), content)?;
        Ok(())
    }

    pub fn render_panel(&self, plugin_id: &str) -> Option<PluginPanel> {
        let plugin = self.plugins.iter().find(|p| p.manifest.id == plugin_id)?;
        if !plugin.enabled {
            return None;
        }
        if !plugin
            .manifest
            .capabilities
            .contains(&PluginCapability::RenderPanel)
        {
            return None;
        }
        None
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

    #[test]
    fn test_plugin_manager_get_plugin() {
        let mut mgr = PluginManager::new();
        assert!(mgr.get_plugin("test").is_none());
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
        assert!(mgr.get_plugin("test").is_some());
        assert!(mgr.get_plugin("other").is_none());
    }

    #[test]
    fn test_plugin_manager_ensure_host() {
        let mut mgr = PluginManager::new();
        mgr.plugins.push(LoadedPlugin {
            manifest: PluginManifest {
                id: "test".to_string(),
                name: "Test".to_string(),
                version: "1.0.0".to_string(),
                description: None,
                author: None,
                capabilities: vec![PluginCapability::ExecuteCommand],
            },
            wasm_path: PathBuf::from("/tmp/test.wasm"),
            enabled: true,
        });
        let host = mgr.ensure_host("test");
        assert!(
            host.blocking_lock()
                .is_capability_allowed(&PluginCapability::ExecuteCommand)
        );
        let host2 = mgr.ensure_host("test");
        assert!(Arc::ptr_eq(&host, &host2));
    }

    #[tokio::test]
    async fn test_plugin_manager_settings_async() {
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
        assert!(mgr.get_plugin_setting("test", "key").await.is_none());
        mgr.set_plugin_setting("test", "key", "value").await;
        assert_eq!(
            mgr.get_plugin_setting("test", "key").await,
            Some("value".to_string())
        );
        let all = mgr.get_all_plugin_settings("test").await;
        assert_eq!(all.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_plugin_manager_fire_event() {
        let mut mgr = PluginManager::new();
        let received = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let received_clone = received.clone();
        mgr.on_event(move |event| {
            received_clone
                .lock()
                .unwrap()
                .push(event.event_type.clone());
        });
        mgr.fire_event(PluginEvent {
            event_type: "test-event".to_string(),
            payload: serde_json::json!({"key": "value"}),
        });
        assert_eq!(&*received.lock().unwrap(), &["test-event"]);
    }

    #[test]
    fn test_plugin_host_all_settings() {
        let mut host = PluginHost::new(&[]);
        assert!(host.all_settings().is_empty());
        host.set_setting("a", "1");
        host.set_setting("b", "2");
        assert_eq!(host.all_settings().len(), 2);
    }

    #[test]
    fn test_plugin_manager_render_panel_disabled() {
        let mut mgr = PluginManager::new();
        mgr.plugins.push(LoadedPlugin {
            manifest: PluginManifest {
                id: "test".to_string(),
                name: "Test".to_string(),
                version: "1.0.0".to_string(),
                description: None,
                author: None,
                capabilities: vec![PluginCapability::RenderPanel],
            },
            wasm_path: PathBuf::from("/tmp/test.wasm"),
            enabled: false,
        });
        assert!(mgr.render_panel("test").is_none());
    }
}
