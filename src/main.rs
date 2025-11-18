use fuuka_bot::FuukaBot;
use tracing::level_filters::LevelFilter;
#[cfg(not(feature = "tokio-console"))]
use tracing_subscriber::fmt::Layer;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .try_from_env()?;
    #[cfg(feature = "tokio-console")]
    let filter = filter.add_directive("tokio=trace,runtime=trace".parse()?);

    #[cfg(feature = "tokio-console")]
    let console_layer = console_subscriber::spawn();
    #[cfg(not(feature = "tokio-console"))]
    let console_layer: Option<Layer<_>> = None;

    #[cfg(feature = "use-journald")]
    let logging_layer = tracing_journald::layer()?;
    #[cfg(not(feature = "use-journald"))]
    let logging_layer = tracing_subscriber::fmt::layer()
        .with_level(true)
        .with_target(true)
        .with_ansi(true)
        .compact();

    tracing_subscriber::registry()
        .with(console_layer)
        .with(logging_layer)
        .with(filter)
        .init();

    FuukaBot::from_config()?
        .enable_media_proxy_if_enabled()?
        .with_key_backups()
        .run()
        .await
}
