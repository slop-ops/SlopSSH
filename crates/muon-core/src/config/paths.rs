//! Application path resolution for config, data, and log directories.
//!
//! Supports both standard OS paths and portable mode (detected via a
//! `portable.marker` file next to the executable).

use std::path::PathBuf;

fn portable_marker() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let dir = exe.parent()?;
    let marker = dir.join("portable.marker");
    if marker.exists() {
        Some(dir.to_path_buf())
    } else {
        None
    }
}

/// Returns the application config directory, creating it if necessary.
///
/// Uses a `config/` subdirectory next to the executable in portable mode,
/// otherwise falls back to the OS-specific config directory (`~/.config/muon-ssh`).
pub fn config_dir() -> anyhow::Result<PathBuf> {
    if let Some(portable_dir) = portable_marker() {
        let dir = portable_dir.join("config");
        if !dir.exists() {
            std::fs::create_dir_all(&dir)?;
        }
        tracing::info!(path = %dir.display(), "Using portable config directory");
        return Ok(dir);
    }
    let dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;
    let dir = dir.join("muon-ssh");
    if !dir.exists() {
        std::fs::create_dir_all(&dir)?;
    }
    Ok(dir)
}

/// Returns the path to `sessions.json` inside the config directory.
pub fn sessions_file() -> anyhow::Result<PathBuf> {
    Ok(config_dir()?.join("sessions.json"))
}

/// Returns the path to `settings.toml` inside the config directory.
pub fn settings_file() -> anyhow::Result<PathBuf> {
    Ok(config_dir()?.join("settings.toml"))
}

/// Returns the path to `snippets.json` inside the config directory.
pub fn snippets_file() -> anyhow::Result<PathBuf> {
    Ok(config_dir()?.join("snippets.json"))
}

/// Returns the path to `tab_state.json` inside the config directory.
pub fn tab_state_file() -> anyhow::Result<PathBuf> {
    Ok(config_dir()?.join("tab_state.json"))
}

/// Returns the log output directory, creating it if necessary.
///
/// Uses a `logs/` subdirectory next to the executable in portable mode,
/// otherwise falls back to the OS-specific local data directory.
pub fn log_dir() -> anyhow::Result<PathBuf> {
    if let Some(portable_dir) = portable_marker() {
        let dir = portable_dir.join("logs");
        if !dir.exists() {
            std::fs::create_dir_all(&dir)?;
        }
        return Ok(dir);
    }
    let dir = dirs::data_local_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine local data directory"))?;
    let dir = dir.join("muon-ssh").join("logs");
    if !dir.exists() {
        std::fs::create_dir_all(&dir)?;
    }
    Ok(dir)
}

/// Returns `true` when running in portable mode.
pub fn is_portable() -> bool {
    portable_marker().is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_dir_returns_valid_path() {
        let dir = config_dir().unwrap();
        assert!(
            dir.to_string_lossy().contains("muon-ssh") || dir.to_string_lossy().contains("config")
        );
    }

    #[test]
    fn test_config_dir_creates_directory() {
        let dir = config_dir().unwrap();
        assert!(dir.exists());
    }

    #[test]
    fn test_sessions_file_ends_with_sessions_json() {
        let path = sessions_file().unwrap();
        assert!(path.to_string_lossy().ends_with("sessions.json"));
    }

    #[test]
    fn test_sessions_file_is_under_config_dir() {
        let base = config_dir().unwrap();
        let sessions = sessions_file().unwrap();
        assert_eq!(sessions.parent(), Some(base.as_path()));
    }

    #[test]
    fn test_settings_file_ends_with_settings_toml() {
        let path = settings_file().unwrap();
        assert!(path.to_string_lossy().ends_with("settings.toml"));
    }

    #[test]
    fn test_settings_file_is_under_config_dir() {
        let base = config_dir().unwrap();
        let settings = settings_file().unwrap();
        assert_eq!(settings.parent(), Some(base.as_path()));
    }

    #[test]
    fn test_snippets_file_ends_with_snippets_json() {
        let path = snippets_file().unwrap();
        assert!(path.to_string_lossy().ends_with("snippets.json"));
    }

    #[test]
    fn test_snippets_file_is_under_config_dir() {
        let base = config_dir().unwrap();
        let snippets = snippets_file().unwrap();
        assert_eq!(snippets.parent(), Some(base.as_path()));
    }

    #[test]
    fn test_log_dir_returns_valid_path() {
        let dir = log_dir().unwrap();
        assert!(
            dir.to_string_lossy().contains("muon-ssh") || dir.to_string_lossy().contains("logs")
        );
    }

    #[test]
    fn test_log_dir_creates_directory() {
        let dir = log_dir().unwrap();
        assert!(dir.exists());
    }

    #[test]
    fn test_config_dir_is_consistent() {
        let dir1 = config_dir().unwrap();
        let dir2 = config_dir().unwrap();
        assert_eq!(dir1, dir2);
    }

    #[test]
    fn test_is_portable_returns_false_in_test_env() {
        assert!(!is_portable());
    }
}
