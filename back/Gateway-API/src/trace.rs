pub fn init_trace() {
    use log::warn;
    use tracing_subscriber::{filter::LevelFilter, EnvFilter};

    let mut warning: Option<String> = None;

    let raw_level = std::env::var("LOG_LEVEL");

    let level_str = match &raw_level {
        Ok(v) => v.as_str(),
        Err(_) => {
            warning = Some("LOG_LEVEL not set, defaulting to info".to_string());
            "info"
        }
    };

    let level_filter = match level_str.to_lowercase().as_str() {
        "trace" => LevelFilter::TRACE,
        "debug" => LevelFilter::DEBUG,
        "info" => LevelFilter::INFO,
        "warn" => LevelFilter::WARN,
        "error" => LevelFilter::ERROR,
        _ => {
            warning = Some(format!(
                "Invalid LOG_LEVEL '{}', defaulting to info",
                level_str
            ));
            LevelFilter::INFO
        }
    };

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(level_filter.into()))
        .init();

    if let Some(msg) = warning {
        warn!("{}", msg);
    }
}
