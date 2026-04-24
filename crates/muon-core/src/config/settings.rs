use serde::{Deserialize, Serialize};

use super::paths;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub language: String,
    pub theme: String,
    pub font_family: String,
    pub font_size: u32,
    pub terminal_scrollback: u32,
    pub terminal_copy_on_select: bool,
    pub show_hidden_files: bool,
    pub default_edit_command: String,
    pub confirm_before_delete: bool,
    pub confirm_before_overwrite: bool,
    pub transfer_parallel_count: u32,
    pub connection_timeout_secs: u64,
    pub keep_alive_interval_secs: u64,
    pub enable_compression: bool,
    pub keyboard_shortcuts: String,
    pub log_level: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            language: "en".to_string(),
            theme: "dark".to_string(),
            font_family: "JetBrains Mono".to_string(),
            font_size: 14,
            terminal_scrollback: 10000,
            terminal_copy_on_select: true,
            show_hidden_files: false,
            default_edit_command: "nano".to_string(),
            confirm_before_delete: true,
            confirm_before_overwrite: true,
            transfer_parallel_count: 4,
            connection_timeout_secs: 30,
            keep_alive_interval_secs: 60,
            enable_compression: false,
            keyboard_shortcuts: String::new(),
            log_level: "info".to_string(),
        }
    }
}

pub struct SettingsManager;

impl SettingsManager {
    pub fn load() -> anyhow::Result<Settings> {
        let path = paths::settings_file()?;
        if !path.exists() {
            let settings = Settings::default();
            Self::save(&settings)?;
            return Ok(settings);
        }
        let content = std::fs::read_to_string(&path)?;
        let settings: Settings = toml::from_str(&content)?;
        Ok(settings)
    }

    pub fn save(settings: &Settings) -> anyhow::Result<()> {
        let path = paths::settings_file()?;
        let content = toml::to_string_pretty(settings)?;
        std::fs::write(&path, content)?;
        Ok(())
    }
}
