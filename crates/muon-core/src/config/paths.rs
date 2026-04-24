use std::path::PathBuf;

pub fn config_dir() -> anyhow::Result<PathBuf> {
    let dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;
    let dir = dir.join("muon-ssh");
    if !dir.exists() {
        std::fs::create_dir_all(&dir)?;
    }
    Ok(dir)
}

pub fn sessions_file() -> anyhow::Result<PathBuf> {
    Ok(config_dir()?.join("sessions.json"))
}

pub fn settings_file() -> anyhow::Result<PathBuf> {
    Ok(config_dir()?.join("settings.toml"))
}

pub fn snippets_file() -> anyhow::Result<PathBuf> {
    Ok(config_dir()?.join("snippets.json"))
}

pub fn log_dir() -> anyhow::Result<PathBuf> {
    let dir = dirs::data_local_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine local data directory"))?;
    let dir = dir.join("muon-ssh").join("logs");
    if !dir.exists() {
        std::fs::create_dir_all(&dir)?;
    }
    Ok(dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_dir_returns_valid_path() {
        let dir = config_dir().unwrap();
        assert!(dir.to_string_lossy().contains("muon-ssh"));
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
        assert!(dir.to_string_lossy().contains("muon-ssh"));
        assert!(dir.to_string_lossy().contains("logs"));
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
}
