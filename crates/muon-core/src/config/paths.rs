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
