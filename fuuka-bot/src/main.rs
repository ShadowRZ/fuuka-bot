use fuuka_bot::FuukaBot;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let filter = EnvFilter::from_default_env()
        .add_directive(LevelFilter::WARN.into())
        .add_directive("fuuka_bot=debug".parse()?)
        .add_directive("matrix_sdk_crypto::backups=error".parse()?);
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_level(true)
        .with_target(true)
        .with_ansi(true)
        .compact()
        .init();

    FuukaBot::from_config()?
        .enable_media_proxy_if_enabled()?
        .with_key_backups()
        .run()
        .await
}
