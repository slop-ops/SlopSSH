//! Application configuration: paths, settings, and editor detection.

pub mod editor;
pub mod paths;
pub mod settings;

/// Top-level application configuration bundle.
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// Resolved config directory path.
    pub config_dir: std::path::PathBuf,
    /// Loaded application settings.
    pub settings: settings::Settings,
}
