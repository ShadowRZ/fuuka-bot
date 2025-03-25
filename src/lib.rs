//! Fuuka Bot Internals for interested.
//!
//! **WARNING: External crate links are broken in the build documentation on GitHub Pages, sorry.**
//!
//! ## User Agent
//!
//! The bot consistently uses the following user agent template:
//!
//! ```text
//! fuuka-bot/<version> (https://github.com/ShadowRZ/fuuka-bot)
//! ```
//!
//! Where `<version>` is the running version of the bot.
pub mod config;
pub mod media_proxy;
pub mod member_changes;
pub mod message;
pub mod render;
pub mod services;
#[doc(hidden)]
#[cfg(feature = "interactive-login")]
pub mod session;
pub mod traits;
pub mod types;

pub use crate::config::Config;
pub use crate::media_proxy::MediaProxy;
pub use crate::member_changes::MembershipHistory;
pub use crate::traits::*;
pub use crate::types::Error;

use matrix_sdk::authentication::matrix::MatrixSession;
use matrix_sdk::ruma::events::room::member::StrippedRoomMemberEvent;
use matrix_sdk::ruma::events::room::tombstone::OriginalSyncRoomTombstoneEvent;
use matrix_sdk::ruma::presence::PresenceState;
use matrix_sdk::{Client, config::SyncSettings};
use pixrs::PixivClient;
use std::path::PathBuf;
use std::sync::Arc;
use thiserror::Error;
use tokio::signal;
use tokio::task::JoinHandle;

static APP_USER_AGENT: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
    " (",
    env!("CARGO_PKG_REPOSITORY"),
    ")"
);

static APP_PRESENCE_TEXT: &str = concat!(
    "Fuuka Bot (v",
    env!("CARGO_PKG_VERSION"),
    ") | ",
    env!("CARGO_PKG_REPOSITORY")
);

/// The bot itself.
pub struct FuukaBot {
    config: Config,
    session: MatrixSession,
    with_key_backups: bool,
    enable_media_proxy_if_enabled: bool,
}

impl FuukaBot {
    pub fn from_config() -> anyhow::Result<Self> {
        use anyhow::Context;

        let config: Config = get_config().context("Getting config failed!")?;

        let cred = get_config_file(CREDENTIALS_FILE)?;

        #[cfg(feature = "interactive-login")]
        if !cred.try_exists()? {
            let session =
                fuuka_bot::session::prompt_for_login_data(&config.matrix.homeserver).await?;
            fs::write(CREDENTIALS_FILE, serde_json::to_string(&session)?)?;
        }

        #[cfg(not(feature = "interactive-login"))]
        if !cred.try_exists()? {
            anyhow::bail!("No credentials files provided!");
        }

        let session = get_credentials().context("Getting credentials failed!")?;

        Ok(Self {
            config,
            session,
            with_key_backups: false,
            enable_media_proxy_if_enabled: false,
        })
    }

    pub fn with_key_backups(mut self) -> Self {
        self.with_key_backups = true;

        self
    }

    pub fn enable_media_proxy_if_enabled(mut self) -> anyhow::Result<Self> {
        self.enable_media_proxy_if_enabled = true;

        Ok(self)
    }

    pub async fn run(self) -> anyhow::Result<()> {
        let http = reqwest::Client::builder()
            .user_agent(APP_USER_AGENT)
            .hickory_dns(true)
            .build()?;
        let pixiv = self.pixiv_client(&http).await?;
        let media_proxy = self.media_proxy(&http).await?;

        let store_path = get_store_path()?;
        let builder = Client::builder()
            .http_client(http.clone())
            .homeserver_url(&self.config.matrix.homeserver)
            .sqlite_store(store_path, None);
        let client = builder.build().await?;
        client.restore_session(self.session).await?;

        // Dispatch CLI args
        let mut args = std::env::args();
        args.next();
        if let Some(arg1) = args.next() {
            match arg1.as_str() {
                #[cfg(feature = "interactive-login")]
                "bootstrap-cross-signing-if-needed" => {
                    Self::bootstrap_cross_signing_if_needed(&client).await?;
                    return Ok(());
                }
                #[cfg(feature = "interactive-login")]
                "bootstrap-cross-signing" => {
                    Self::bootstrap_cross_signing(&client).await?;
                    return Ok(());
                }
                #[cfg(feature = "interactive-login")]
                "reset-cross-signing" => {
                    Self::reset_cross_signing(&client).await?;
                    return Ok(());
                }
                _ => {
                    println!("Unknown command!");
                    return Ok(());
                }
            }
        }

        if self.with_key_backups {
            Self::enable_key_backups(&client).await?;
        }

        let prefix = self.config.command.prefix.clone();
        let (_, config) = tokio::sync::watch::channel(self.config);

        let injected = self::message::Injected {
            config,
            prefix,
            http,
            pixiv,
            media_proxy,
        };

        client.add_event_handler_context(injected);
        let task: JoinHandle<()> = tokio::spawn(async move {
            tokio::select! {
                _ = async {
                    while let Err(e) = sync(&client).await {
                        use tokio::time::{sleep, Duration};
                        tracing::error!("Unexpected error happened, retrying in 10s: {e:#}");
                        sleep(Duration::from_secs(10)).await;
                    }
                } => {},
                _ = graceful_shutdown_future() => {
                    tracing::info!("Bye!");
                },
            }
        });

        Ok(task.await?)
    }

    async fn pixiv_client(
        &self,
        http: &reqwest::Client,
    ) -> anyhow::Result<Option<Arc<PixivClient>>> {
        let Some(ref token) = self.config.pixiv.token else {
            return Ok(None);
        };
        Ok(Some(Arc::new(PixivClient::from_client(token, http).await?)))
    }

    async fn media_proxy(&self, http: &reqwest::Client) -> anyhow::Result<Option<Arc<MediaProxy>>> {
        if !self.enable_media_proxy_if_enabled {
            return Ok(None);
        }

        match &self.config.media_proxy {
            Some(config) => {
                use anyhow::Context;

                let jwk = get_jwk_token().context("Locate JWK file failed")?;
                let media_proxy = MediaProxy::new(
                    self.config.matrix.homeserver.clone(),
                    self.session.tokens.access_token.clone(),
                    jwk,
                    http,
                )?;

                let addr = &config.listen;
                let router = media_proxy.router();
                let listener = tokio::net::TcpListener::bind(addr).await?;
                tokio::spawn(async move {
                    axum::serve(listener, router)
                        .with_graceful_shutdown(graceful_shutdown_future())
                        .await
                });
                Ok(Some(Arc::new(media_proxy)))
            }
            None => Ok(None),
        }
    }

    async fn enable_key_backups(client: &matrix_sdk::Client) -> anyhow::Result<()> {
        let backup = client.encryption().backups();

        if backup.are_enabled().await && backup.exists_on_server().await? {
            tracing::debug!(
                "Bot has an existing server key backup that is valid, skipping recovery provision."
            );
            return Ok(());
        }

        let recovery = client.encryption().recovery();
        let enable = recovery.enable().wait_for_backups_to_upload();

        let mut progress = enable.subscribe_to_progress();

        tokio::spawn(async move {
            use futures_util::StreamExt;
            use matrix_sdk::encryption::recovery::EnableProgress;

            while let Some(update) = progress.next().await {
                let Ok(update) = update else {
                    panic!("Update to the enable progress lagged");
                };

                match update {
                    EnableProgress::CreatingBackup => {
                        tracing::debug!("Creating a new backup");
                    }
                    EnableProgress::CreatingRecoveryKey => {
                        tracing::debug!("Creating a new recovery key");
                    }
                    EnableProgress::Done { .. } => {
                        tracing::debug!("Recovery has been enabled");
                        break;
                    }
                    _ => (),
                }
            }
        });

        match enable.await {
            Ok(key) => tracing::info!("The recovery key is: {key}"),
            Err(e) => tracing::warn!("Error while enabling backup: {e:#}"),
        }

        Ok(())
    }

    /// Prepares the bootstrap cross signing key if needed.
    #[cfg(feature = "interactive-login")]
    async fn bootstrap_cross_signing_if_needed(client: &matrix_sdk::Client) -> anyhow::Result<()> {
        use anyhow::Context;
        use matrix_sdk::ruma::api::client::uiaa;
        use rpassword::read_password;

        if let Err(e) = client
            .encryption()
            .bootstrap_cross_signing_if_needed(None)
            .await
        {
            if let Some(response) = e.as_uiaa_response() {
                use std::io::Write;

                print!("Enter password for preparing cross signing: ");
                std::io::stdout().flush()?;
                let password = read_password()?;
                let mut password = uiaa::Password::new(
                    uiaa::UserIdentifier::UserIdOrLocalpart(client.user_id().unwrap().to_string()),
                    password,
                );
                password.session = response.session.clone();

                client
                    .encryption()
                    .bootstrap_cross_signing(Some(uiaa::AuthData::Password(password)))
                    .await
                    .context("Couldn't bootstrap cross signing")?
            } else {
                anyhow::bail!("Error during cross signing bootstrap {:#?}", e);
            }
        }

        Ok(())
    }

    /// Prepares the bootstrap cross signing key.
    #[cfg(feature = "interactive-login")]
    async fn bootstrap_cross_signing(client: &matrix_sdk::Client) -> anyhow::Result<()> {
        use anyhow::Context;
        use matrix_sdk::ruma::api::client::uiaa;
        use rpassword::read_password;

        if let Err(e) = client.encryption().bootstrap_cross_signing(None).await {
            if let Some(response) = e.as_uiaa_response() {
                use std::io::Write;

                print!("Enter password for preparing cross signing: ");
                std::io::stdout().flush()?;
                let password = read_password()?;
                let mut password = uiaa::Password::new(
                    uiaa::UserIdentifier::UserIdOrLocalpart(client.user_id().unwrap().to_string()),
                    password,
                );
                password.session = response.session.clone();

                client
                    .encryption()
                    .bootstrap_cross_signing(Some(uiaa::AuthData::Password(password)))
                    .await
                    .context("Couldn't bootstrap cross signing")?
            } else {
                anyhow::bail!("Error during cross signing bootstrap {:#?}", e);
            }
        }

        Ok(())
    }

    /// Resets the bootstrap cross signing key.
    #[cfg(feature = "interactive-login")]
    async fn reset_cross_signing(client: &matrix_sdk::Client) -> anyhow::Result<()> {
        use matrix_sdk::encryption::CrossSigningResetAuthType;

        if let Some(handle) = client.encryption().reset_cross_signing().await? {
            match handle.auth_type() {
                CrossSigningResetAuthType::Uiaa(uiaa) => {
                    use matrix_sdk::ruma::api::client::uiaa;
                    use rpassword::read_password;
                    use std::io::Write;

                    print!("Enter password for resetting cross signing: ");
                    std::io::stdout().flush()?;
                    let password = read_password()?;
                    let mut password = uiaa::Password::new(
                        uiaa::UserIdentifier::UserIdOrLocalpart(
                            client.user_id().unwrap().to_string(),
                        ),
                        password,
                    );
                    password.session = uiaa.session.clone();

                    handle
                        .auth(Some(uiaa::AuthData::Password(password)))
                        .await?;
                }
                CrossSigningResetAuthType::Oidc(o) => {
                    tracing::info!(
                        "To reset your end-to-end encryption cross-signing identity, \
                            you first need to approve it at {}",
                        o.approval_url
                    );
                    handle.auth(None).await?;
                }
            }
        }

        Ok(())
    }
}

/// A sharable graceful shutdown signal.
pub async fn graceful_shutdown_future() {
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
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

static CREDENTIALS_FILE: &str = "credentials.json";
static CONFIG_FILE: &str = "fuuka-bot.toml";
static JWK_TOKEN_FILE: &str = "fuuka-bot.jwk.json";

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

fn get_jwk_token() -> anyhow::Result<jose_jwk::Jwk> {
    let file = get_config_file(JWK_TOKEN_FILE)?;

    let contents = std::fs::read_to_string(file)?;
    let jwk = serde_json::from_str::<jose_jwk::Jwk>(&contents)?;
    Ok(jwk)
}

fn get_credentials() -> anyhow::Result<MatrixSession> {
    let file = get_config_file(CREDENTIALS_FILE)?;

    let contents = std::fs::read_to_string(file)?;
    let session = serde_json::from_str::<MatrixSession>(&contents)?;
    Ok(session)
}

fn get_config() -> anyhow::Result<Config> {
    let file = get_config_file(CONFIG_FILE)?;

    let contents = std::fs::read_to_string(file)?;
    let config = toml::from_str::<Config>(&contents)?;
    Ok(config)
}

fn get_store_path() -> anyhow::Result<PathBuf> {
    static ENV_FUUKA_BOT_STATE_DIRECTORY: &str = "FUUKA_BOT_STATE_DIRECTORY";
    static ENV_STATE_DIRECTORY: &str = "STATE_DIRECTORY";

    static SQLITE_STORE_PATH: &str = "store";

    let dir = std::env::var(ENV_FUUKA_BOT_STATE_DIRECTORY)
        .ok()
        .or_else(|| std::env::var(ENV_STATE_DIRECTORY).ok());

    let mut path = PathBuf::new();
    if let Some(dir) = dir {
        path.push(dir);
    }
    path.push(SQLITE_STORE_PATH);

    Ok(path)
}

async fn ensure_self_device_verified(client: &matrix_sdk::Client) -> anyhow::Result<()> {
    let encryption = client.encryption();
    let has_keys = encryption
        .cross_signing_status()
        .await
        .map(|status| status.has_self_signing && status.has_master)
        .unwrap_or_default();

    if !has_keys {
        tracing::warn!("No self signing key to sign this own device!");
        return Ok(());
    }

    if let Some(device) = encryption.get_own_device().await? {
        if !device.is_cross_signed_by_owner() {
            device.verify().await?
        }
    }

    Ok(())
}

async fn sync(client: &matrix_sdk::Client) -> anyhow::Result<()> {
    let user_id = client.user_id();
    let homeserver = client.homeserver();
    tracing::info!(user_id = ?user_id, homeserver = %homeserver, "Initial sync beginning...");
    let response = client
        .sync_once(SyncSettings::default().set_presence(PresenceState::Online))
        .await?;
    tracing::info!(user_id = ?user_id, homeserver = %homeserver, "Initial sync completed.");
    // Print some info regarding this device.
    {
        let encryption = client.encryption();
        let cross_signing_status = encryption.cross_signing_status().await;
        if let Some(device) = encryption.get_own_device().await? {
            let device_id = device.device_id();
            tracing::debug!(
                cross_signing_status = ?cross_signing_status,
                is_cross_signed_by_owner = device.is_cross_signed_by_owner(),
                is_verified = device.is_verified(),
                is_verified_with_cross_signing = device.is_verified_with_cross_signing(),
                "Own device ID: {device_id}"
            );
        }
    }

    let next_batch = response.next_batch;

    if let Err(e) = ensure_self_device_verified(client).await {
        tracing::warn!("Failed to ensure this device is verified: {e:#}");
    }
    let settings = SyncSettings::default()
        .token(next_batch)
        .set_presence(PresenceState::Online);
    use matrix_sdk::ruma::api::client::presence::set_presence::v3::Request;
    if let Some(user_id) = client.user_id() {
        let mut presence = Request::new(user_id.into(), PresenceState::Online);
        presence.status_msg = Some(APP_PRESENCE_TEXT.to_string());
        if let Err(e) = client.send(presence).await {
            tracing::warn!("Failed to set presence: {e:#}");
        }
    }

    let h1 = client.add_event_handler(crate::message::on_sync_message);
    let h2 = client.add_event_handler(crate::on_stripped_member);
    let h3 = client.add_event_handler(crate::on_room_replace);

    if let Err(e) = client.sync(settings).await {
        client.remove_event_handler(h1);
        client.remove_event_handler(h2);
        client.remove_event_handler(h3);
        return Err(e.into());
    }
    Ok(())
}

/// Called when a member event is from an invited room.
pub async fn on_stripped_member(
    ev: StrippedRoomMemberEvent,
    room: matrix_sdk::Room,
    client: matrix_sdk::Client,
) {
    // Ignore state events not for ourselves.
    if ev.state_key != client.user_id().unwrap() {
        return;
    }

    tokio::spawn(async move {
        let room_id = room.room_id();
        tracing::info!("Autojoining room {}", room_id);
        let mut delay = 2;
        while let Err(e) = room.join().await {
            use tokio::time::{Duration, sleep};
            tracing::warn!("Failed to join room {room_id} ({e:#}), retrying in {delay}s");
            sleep(Duration::from_secs(delay)).await;
            delay *= 2;

            if delay > 3600 {
                tracing::error!("Can't join room {room_id} ({e:#})");
                break;
            }
        }
    });
}

/// Called when we have a tombstone event.
pub async fn on_room_replace(
    ev: OriginalSyncRoomTombstoneEvent,
    room: matrix_sdk::Room,
    client: matrix_sdk::Client,
) {
    tokio::spawn(async move {
        let room_id = ev.content.replacement_room;
        tracing::info!("Room replaced, Autojoining new room {}", room_id);
        let mut delay = 2;
        while let Err(e) = client.join_room_by_id(&room_id).await {
            use tokio::time::{Duration, sleep};
            tracing::warn!("Failed to join room {room_id} ({e:#}), retrying in {delay}s");
            sleep(Duration::from_secs(delay)).await;
            delay *= 2;

            if delay > 3600 {
                tracing::error!("Can't join room {room_id} ({e:#})");
                break;
            }
        }
        if let Some(room) = client.get_room(room.room_id()) {
            tokio::spawn(async move {
                if let Err(e) = room.leave().await {
                    tracing::error!("Can't leave the original room {} ({e:#})", room.room_id());
                }
            });
        }
    });
}
