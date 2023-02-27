use tracing_log::LogTracer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Registry};

pub fn init() {
    LogTracer::init().unwrap();
    let format = tracing_subscriber::fmt::format()
        .without_time()
        .with_target(true);
    let stdout_log = tracing_subscriber::fmt::layer().event_format(format);
    let subscriber = Registry::default()
        .with(EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "INFO".to_owned()),
        ))
        .with(stdout_log);
    tracing::subscriber::set_global_default(subscriber).unwrap();
}
