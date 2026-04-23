use tracing_subscriber::EnvFilter;

pub fn init(level: &str) {
    let _log_dir = match crate::config::paths::log_dir() {
        Ok(dir) => dir,
        Err(_) => return,
    };

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level));

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_writer(std::io::stderr)
        .init();

    tracing::info!("Muon SSH logging initialized (level={})", level);
}
