pub mod paths;
pub mod settings;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub config_dir: std::path::PathBuf,
    pub settings: settings::Settings,
}
