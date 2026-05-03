//! Plugin API type definitions: manifests, capabilities, events, and panels.

use serde::{Deserialize, Serialize};

/// Describes a plugin's identity and capabilities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    /// Unique plugin identifier.
    pub id: String,
    /// Human-readable plugin name.
    pub name: String,
    /// Semantic version string.
    pub version: String,
    /// Optional plugin description.
    pub description: Option<String>,
    /// Optional plugin author.
    pub author: Option<String>,
    /// List of capabilities the plugin requests.
    pub capabilities: Vec<PluginCapability>,
}

/// Capabilities a plugin can request access to.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginCapability {
    /// Execute shell commands on the host.
    ExecuteCommand,
    /// Read plugin settings.
    ReadSetting,
    /// Show desktop notifications.
    ShowNotification,
    /// Receive session connect events.
    OnSessionConnect,
    /// Receive session disconnect events.
    OnSessionDisconnect,
    /// Render a custom UI panel.
    RenderPanel,
}

/// A custom UI panel provided by a plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginPanel {
    /// Panel title shown in the UI.
    pub title: String,
    /// Content type of the panel body.
    pub content_type: PanelContentType,
    /// Panel body content.
    pub content: String,
}

/// Supported content types for plugin panels.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PanelContentType {
    /// HTML content.
    Html,
    /// JSON data.
    Json,
    /// Markdown text.
    Markdown,
    /// Plain text.
    Text,
}

/// An event dispatched to or from a plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginEvent {
    /// Event type identifier.
    pub event_type: String,
    /// JSON payload associated with the event.
    pub payload: serde_json::Value,
}
