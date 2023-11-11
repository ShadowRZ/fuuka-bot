use fuuka_bot::FuukaBot;
use fuuka_bot::FuukaBotConfig;
use matrix_sdk::matrix_auth::MatrixSession;
use matrix_sdk::Client;
use reqwest::Url;
use rpassword::read_password;
use std::env;
use std::fs;
use std::io;
use std::io::Write;

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

fn get_session() -> Option<MatrixSession> {
    if let Ok(contents) = fs::read_to_string(SESSION_JSON_FILE) {
        if let Ok(session) = serde_json::from_str::<MatrixSession>(&contents) {
            Some(session)
        } else {
            None
        }
    } else {
        None
    }
}

fn get_config() -> Option<FuukaBotConfig> {
    if let Ok(contents) = fs::read_to_string(CONFIG_FILE) {
        if let Ok(config) = toml::from_str::<FuukaBotConfig>(&contents) {
            Some(config)
        } else {
            None
        }
    } else {
        None
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let config: FuukaBotConfig = get_config().expect("Getting config failed!");

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

    let session = get_session().expect("Getting session failed!");

    let bot = FuukaBot::new(config, session).await?;
    bot.run().await
}
