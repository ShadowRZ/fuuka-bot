use tracing::level_filters::LevelFilter;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();
    let logger = tracing_subscriber::fmt::layer()
        .with_level(true)
        .with_target(true)
        .with_ansi(true)
        .compact();

    tracing_subscriber::registry()
        .with(logger)
        .with(filter)
        .init();

    fuuka_bot::builder()
        .with_key_backups()
        .with_optional_media_proxy()
        .build()?
        .run()
        .await
}
