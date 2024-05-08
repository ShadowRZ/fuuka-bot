use std::io::{self, Write};

use matrix_sdk::{matrix_auth::MatrixSession, Client};
use rpassword::read_password;
use url::Url;

async fn login(homeserver: &Url, username: &str, password: &str) -> anyhow::Result<MatrixSession> {
    let client = Client::new(homeserver.clone()).await?;
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
    Ok(session)
}

pub async fn prompt_for_login_data(homeserver: &Url) -> anyhow::Result<MatrixSession> {
    println!("Homeserver is: {}", homeserver);
    print!("Enter username: ");
    io::stdout().flush()?;
    let mut username = String::new();
    io::stdin().read_line(&mut username)?;
    let username = username.trim();
    print!("Enter password: ");
    io::stdout().flush()?;
    let password = read_password()?;
    let session = login(homeserver, username, &password).await?;
    println!("Session stored, running......");
    Ok(session)
}
