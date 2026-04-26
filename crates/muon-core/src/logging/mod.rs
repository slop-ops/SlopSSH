use tracing_subscriber::EnvFilter;

pub fn init(level: &str) {
    let log_dir = match crate::config::paths::log_dir() {
        Ok(dir) => dir,
        Err(e) => {
            init_stderr_only(level);
            tracing::warn!(error = %e, "Failed to create log directory, using stderr only");
            return;
        }
    };

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level));

    let file_appender = tracing_appender::rolling::daily(log_dir, "muon-ssh.log");
    let (file_writer, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_writer(tracing_subscriber::fmt::writer::Tee::new(
            std::io::stderr,
            file_writer,
        ))
        .init();

    std::mem::forget(_guard);

    tracing::info!("Muon SSH logging initialized (level={})", level);
}

fn init_stderr_only(level: &str) {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level));

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_writer(std::io::stderr)
        .init();

    tracing::info!(
        "Muon SSH logging initialized (level={}, stderr only)",
        level
    );
}
