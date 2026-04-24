use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub capabilities: Vec<PluginCapability>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginCapability {
    ExecuteCommand,
    ReadSetting,
    ShowNotification,
    OnSessionConnect,
    OnSessionDisconnect,
    RenderPanel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginPanel {
    pub title: String,
    pub content_type: PanelContentType,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PanelContentType {
    Html,
    Json,
    Markdown,
    Text,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginEvent {
    pub event_type: String,
    pub payload: serde_json::Value,
}
