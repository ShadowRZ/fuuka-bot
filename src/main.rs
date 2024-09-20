use anyhow::Context;
use fuuka_bot::Config;
use fuuka_bot::FuukaBot;
use matrix_sdk::matrix_auth::MatrixSession;
use std::fs;
use std::path::PathBuf;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

static CREDENTIALS_FILE: &str = "credentials.json";
static CONFIG_FILE: &str = "fuuka-bot.toml";

fn get_config_file(file: &'static str) -> anyhow::Result<PathBuf> {
    static ENV_FUUKA_BOT_CONFIGURATION_DIRECTORY: &str = "FUUKA_BOT_CONFIGURATION_DIRECTORY";
    static ENV_CONFIGURATION_DIRECTORY: &str = "CONFIGURATION_DIRECTORY";

    let dir = std::env::var(ENV_FUUKA_BOT_CONFIGURATION_DIRECTORY)
        .ok()
        .or_else(|| std::env::var(ENV_CONFIGURATION_DIRECTORY).ok());

    let mut path = PathBuf::new();
    if let Some(dir) = dir {
        path.push(dir);
    }
    path.push(file);

    Ok(path)
}

fn get_credentials() -> anyhow::Result<MatrixSession> {
    let file = get_config_file(CREDENTIALS_FILE)?;

    let contents = fs::read_to_string(file)?;
    let session = serde_json::from_str::<MatrixSession>(&contents)?;
    Ok(session)
}

fn get_config() -> anyhow::Result<Config> {
    let file = get_config_file(CONFIG_FILE)?;

    let contents = fs::read_to_string(file)?;
    let config = toml::from_str::<Config>(&contents)?;
    Ok(config)
}

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

    let config: Config = get_config().context("Getting config failed!")?;

    let cred = get_config_file(CREDENTIALS_FILE)?;

    #[cfg(feature = "interactive-login")]
    if !cred.try_exists()? {
        let session = fuuka_bot::session::prompt_for_login_data(&config.matrix.homeserver).await?;
        fs::write(CREDENTIALS_FILE, serde_json::to_string(&session)?)?;
    }

    #[cfg(not(feature = "interactive-login"))]
    if !cred.try_exists()? {
        anyhow::bail!("No credentials files provided!");
    }

    let session = get_credentials().context("Getting credentials failed!")?;

    FuukaBot::new(config, session)
        .await?
        .with_shutdown()
        .enable_recovery()
        .await?
        .run()
        .await
}
