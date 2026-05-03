//! Application settings with TOML persistence and validation.

use serde::{Deserialize, Serialize};

use super::paths;

/// Global application settings persisted as `settings.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// UI language code (e.g. `"en"`, `"de"`).
    pub language: String,
    /// Theme name (`"dark"` or `"light"`).
    pub theme: String,
    /// Terminal font family.
    pub font_family: String,
    /// Terminal font size in points.
    pub font_size: u32,
    /// Maximum number of lines kept in the terminal scrollback buffer.
    pub terminal_scrollback: u32,
    /// Whether selected text is automatically copied to the clipboard.
    pub terminal_copy_on_select: bool,
    /// Whether to show hidden files in the file browser.
    pub show_hidden_files: bool,
    /// Default editor command for remote file editing.
    pub default_edit_command: String,
    /// External editor binary path (overrides `default_edit_command` when set).
    pub external_editor: String,
    /// Whether to prompt before deleting files.
    pub confirm_before_delete: bool,
    /// Whether to prompt before overwriting files.
    pub confirm_before_overwrite: bool,
    /// Number of parallel connections for file transfers.
    pub transfer_parallel_count: u32,
    /// SSH connection timeout in seconds.
    pub connection_timeout_secs: u64,
    /// SSH keep-alive interval in seconds.
    pub keep_alive_interval_secs: u64,
    /// Whether to enable SSH compression.
    pub enable_compression: bool,
    /// Keyboard shortcuts profile name (JSON).
    pub keyboard_shortcuts: String,
    /// Log verbosity level (`"trace"`, `"debug"`, `"info"`, `"warn"`, `"error"`).
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
            external_editor: String::new(),
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

impl Settings {
    /// Clamps all fields to their allowed ranges and resets invalid values to defaults.
    pub fn validate(&mut self) {
        if self.font_size == 0 {
            self.font_size = 14;
        }
        self.font_size = self.font_size.clamp(6, 72);

        if self.terminal_scrollback == 0 {
            self.terminal_scrollback = 10000;
        }
        self.terminal_scrollback = self.terminal_scrollback.clamp(100, 1_000_000);

        if self.transfer_parallel_count == 0 {
            self.transfer_parallel_count = 4;
        }
        self.transfer_parallel_count = self.transfer_parallel_count.clamp(1, 16);

        if self.connection_timeout_secs == 0 {
            self.connection_timeout_secs = 30;
        }
        self.connection_timeout_secs = self.connection_timeout_secs.clamp(5, 300);

        if self.keep_alive_interval_secs == 0 {
            self.keep_alive_interval_secs = 60;
        }
        self.keep_alive_interval_secs = self.keep_alive_interval_secs.clamp(10, 3600);

        let valid_log_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_log_levels.contains(&self.log_level.as_str()) {
            self.log_level = "info".to_string();
        }

        let valid_themes = ["dark", "light"];
        if !valid_themes.contains(&self.theme.as_str()) {
            self.theme = "dark".to_string();
        }

        if self.language.is_empty() {
            self.language = "en".to_string();
        }

        if self.font_family.trim().is_empty() {
            self.font_family = "JetBrains Mono".to_string();
        }
    }
}

/// Load/save helper for [`Settings`].
pub struct SettingsManager;

impl SettingsManager {
    /// Loads settings from disk, creating the file with defaults if it does not exist.
    pub fn load() -> anyhow::Result<Settings> {
        let path = paths::settings_file()?;
        if !path.exists() {
            let mut settings = Settings::default();
            Self::save(&mut settings)?;
            return Ok(settings);
        }
        let content = std::fs::read_to_string(&path)?;
        let mut settings: Settings = toml::from_str(&content)?;
        settings.validate();
        Ok(settings)
    }

    /// Validates and persists settings to disk.
    pub fn save(settings: &mut Settings) -> anyhow::Result<()> {
        settings.validate();
        let path = paths::settings_file()?;
        let content = toml::to_string_pretty(settings)?;
        std::fs::write(&path, content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = Settings::default();
        assert_eq!(settings.language, "en");
        assert_eq!(settings.theme, "dark");
        assert_eq!(settings.font_family, "JetBrains Mono");
        assert_eq!(settings.font_size, 14);
        assert_eq!(settings.terminal_scrollback, 10000);
        assert!(settings.terminal_copy_on_select);
        assert!(!settings.show_hidden_files);
        assert!(settings.confirm_before_delete);
        assert!(settings.confirm_before_overwrite);
        assert_eq!(settings.transfer_parallel_count, 4);
        assert_eq!(settings.connection_timeout_secs, 30);
        assert_eq!(settings.keep_alive_interval_secs, 60);
        assert!(!settings.enable_compression);
    }

    #[test]
    fn test_settings_serialize_deserialize() {
        let settings = Settings::default();
        let toml_str = toml::to_string_pretty(&settings).unwrap();
        let parsed: Settings = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.language, settings.language);
        assert_eq!(parsed.theme, settings.theme);
        assert_eq!(parsed.font_size, settings.font_size);
        assert_eq!(parsed.enable_compression, settings.enable_compression);
    }

    #[test]
    fn test_settings_missing_fields_use_defaults() {
        let toml_str = r#"
language = "de"
theme = "light"
"#;
        let parsed: Settings = toml::from_str(toml_str).unwrap();
        assert_eq!(parsed.language, "de");
        assert_eq!(parsed.theme, "light");
        assert_eq!(parsed.font_family, "JetBrains Mono");
        assert_eq!(parsed.font_size, 14);
    }

    #[test]
    fn test_validate_clamps_font_size() {
        let mut s = Settings {
            font_size: 0,
            ..Default::default()
        };
        s.validate();
        assert_eq!(s.font_size, 14);

        s.font_size = 100;
        s.validate();
        assert_eq!(s.font_size, 72);
    }

    #[test]
    fn test_validate_clamps_scrollback() {
        let mut s = Settings {
            terminal_scrollback: 0,
            ..Default::default()
        };
        s.validate();
        assert_eq!(s.terminal_scrollback, 10000);

        s.terminal_scrollback = 50;
        s.validate();
        assert_eq!(s.terminal_scrollback, 100);
    }

    #[test]
    fn test_validate_clamps_timeout() {
        let mut s = Settings {
            connection_timeout_secs: 0,
            ..Default::default()
        };
        s.validate();
        assert_eq!(s.connection_timeout_secs, 30);

        s.connection_timeout_secs = 500;
        s.validate();
        assert_eq!(s.connection_timeout_secs, 300);
    }

    #[test]
    fn test_validate_clamps_keep_alive() {
        let mut s = Settings {
            keep_alive_interval_secs: 0,
            ..Default::default()
        };
        s.validate();
        assert_eq!(s.keep_alive_interval_secs, 60);
    }

    #[test]
    fn test_validate_clamps_parallel_count() {
        let mut s = Settings {
            transfer_parallel_count: 0,
            ..Default::default()
        };
        s.validate();
        assert_eq!(s.transfer_parallel_count, 4);

        s.transfer_parallel_count = 50;
        s.validate();
        assert_eq!(s.transfer_parallel_count, 16);
    }

    #[test]
    fn test_validate_fixes_invalid_log_level() {
        let mut s = Settings {
            log_level: "verbose".to_string(),
            ..Default::default()
        };
        s.validate();
        assert_eq!(s.log_level, "info");
    }

    #[test]
    fn test_validate_fixes_invalid_theme() {
        let mut s = Settings {
            theme: "blue".to_string(),
            ..Default::default()
        };
        s.validate();
        assert_eq!(s.theme, "dark");
    }

    #[test]
    fn test_validate_fixes_empty_language() {
        let mut s = Settings {
            language: "".to_string(),
            ..Default::default()
        };
        s.validate();
        assert_eq!(s.language, "en");
    }

    #[test]
    fn test_validate_fixes_empty_font_family() {
        let mut s = Settings {
            font_family: "   ".to_string(),
            ..Default::default()
        };
        s.validate();
        assert_eq!(s.font_family, "JetBrains Mono");
    }

    #[test]
    fn test_validate_preserves_valid_settings() {
        let mut s = Settings::default();
        s.validate();
        assert_eq!(s.font_size, 14);
        assert_eq!(s.language, "en");
        assert_eq!(s.theme, "dark");
        assert_eq!(s.log_level, "info");
    }
}
