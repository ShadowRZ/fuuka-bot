use anyhow::Context;
use fuuka_bot::FuukaBot;
use fuuka_bot::FuukaBotConfig;
use matrix_sdk::matrix_auth::MatrixSession;
use matrix_sdk::Client;
use reqwest::Url;
use rpassword::read_password;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::filter;
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use std::env;
use std::fs;
use std::io;
use std::io::Write;
use tokio::signal;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

static SESSION_JSON_FILE: &str = "credentials.json";
static CONFIG_FILE: &str = "fuuka-bot.toml";

async fn save_login_session(
    homeserver: &str,
    username: &str,
    password: &str,
) -> anyhow::Result<()> {
    let url = Url::parse(homeserver)?;
    let client = Client::new(url).await?;
    let client_auth = client.matrix_auth();

    loop {
        match client_auth.login_username(username, password).await {
            Ok(_) => {
                println!("Logged in as {username}");
                break;
            }
            Err(error) => {
                println!("Error logging in: {error}");
                println!("Trying again......\n");
            }
        }
    }
    let session = client_auth
        .session()
        .expect("A logged-in client should have a session");
    fs::write(SESSION_JSON_FILE, serde_json::to_string(&session)?)?;
    Ok(())
}

fn get_session() -> anyhow::Result<MatrixSession> {
    let contents = fs::read_to_string(SESSION_JSON_FILE)?;
    let session = serde_json::from_str::<MatrixSession>(&contents)?;
    Ok(session)
}

fn get_config() -> anyhow::Result<FuukaBotConfig> {
    let contents = fs::read_to_string(CONFIG_FILE)?;
    let config = toml::from_str::<FuukaBotConfig>(&contents)?;
    Ok(config)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let filter = EnvFilter::from_default_env()
        .add_directive(LevelFilter::WARN.into())
        .add_directive("fuuka_bot=debug".parse()?);
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_level(true)
        .with_target(true)
        .with_ansi(true)
        .compact()
        .init();

    let config: FuukaBotConfig = get_config().context("Getting config failed!")?;

    if let Some(arg1) = env::args().nth(1) {
        if arg1 == "login" {
            println!("Homeserver is: {}", config.homeserver_url);
            print!("Enter username: ");
            io::stdout().flush()?;
            let mut username = String::new();
            io::stdin().read_line(&mut username)?;
            let username = username.trim();
            print!("Enter password: ");
            io::stdout().flush()?;
            let password = read_password()?;
            save_login_session(&config.homeserver_url, username, &password).await?;
            println!("Session stored, you can run the bot now.");
            return Ok(());
        }
    }

    let session = get_session().context("Getting session failed!")?;

    let cts = CancellationToken::new();
    let bot_cts = cts.clone();
    spawn_shutdown_handler(cts).await;

    let bot = FuukaBot::new(config, session).await?;
    let task: JoinHandle<anyhow::Result<()>> = tokio::spawn(async move {
        tokio::select! {
            _ = bot_cts.cancelled() => {
                tracing::info!("Shutdown signal received, starting graceful shutdown");
                Ok(())
            }
            _ = bot.run() => {
                Ok(())
            }
        }
    });

    task.await?
}

async fn spawn_shutdown_handler(cts: CancellationToken) {
    tokio::spawn(async move {
        let ctrl_c = async {
            signal::ctrl_c()
                .await
                .expect("failed to install Ctrl+C handler");
        };

        #[cfg(unix)]
        let terminate = async {
            signal::unix::signal(signal::unix::SignalKind::terminate())
                .expect("failed to install signal handler")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => {
                cts.cancel();
            },
            _ = terminate => {
                cts.cancel();
            },
        }
    });
}
