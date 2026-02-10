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
pub mod format;
pub mod matrix;
pub mod media_proxy;
pub mod member_changes;
pub mod message;
pub mod services;
pub mod traits;
pub mod types;

pub use crate::config::Config;
pub use crate::media_proxy::MediaProxy;
pub use crate::member_changes::MembershipHistory;
pub use crate::traits::*;
pub use crate::types::Error;

use clap::Parser;
use matrix_sdk::authentication::matrix::MatrixSession;
use matrix_sdk::config::RequestConfig;
use matrix_sdk::config::SyncSettings;
use matrix_sdk::ruma::OwnedRoomOrAliasId;
use matrix_sdk::ruma::events::room::ImageInfo;
use matrix_sdk::ruma::events::room::member::StrippedRoomMemberEvent;
use matrix_sdk::ruma::events::room::tombstone::OriginalSyncRoomTombstoneEvent;
use matrix_sdk::ruma::presence::PresenceState;
use pixrs::PixivClient;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::signal;
use tokio::task::JoinHandle;
use tracing::Instrument;

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

static APP_DEFAULT_TIMEOUT: Option<Duration> = Some(Duration::from_secs(300));

#[derive(Debug, clap::Parser)]
#[command(disable_help_subcommand = true)]
pub struct Args {
    #[command(subcommand)]
    command: Option<Subcommand>,
}

#[derive(Debug, clap::Subcommand)]
pub enum Subcommand {
    /// Create and upload a new cross signing identity.
    BootstrapCrossSigning {
        /// Only perform this action if that has not been done yet.
        #[arg(long)]
        if_needed: bool,
    },
    /// Reset the cross-signing keys.
    ResetCrossSigning,
    /// Recover all the secrets from the homeserver.
    RecoverCrossSigning,
    /// Create a new secret store.
    CreateSecretStore,
    /// Create a new backup version, encrypted with a new backup recovery key.
    NewBackup,
}

/// Builder for the bot.
#[derive(Default)]
pub struct Builder {
    with_key_backups: bool,
    with_optional_media_proxy: bool,
}

pub fn builder() -> Builder {
    Default::default()
}

impl Builder {
    pub fn with_key_backups(mut self) -> Self {
        self.with_key_backups = true;

        self
    }

    pub fn with_optional_media_proxy(mut self) -> Self {
        self.with_optional_media_proxy = true;

        self
    }

    pub fn build(self) -> anyhow::Result<crate::Client> {
        let args = Args::parse();
        use anyhow::Context;

        let cred = get_config_file(CREDENTIALS_FILE)?;
        if !cred.try_exists()? {
            anyhow::bail!("No credentials files provided!");
        }

        let session = get_credentials().context("Getting credentials failed!")?;

        let config: Config = get_config().context("Getting config failed!")?;

        let http = reqwest::Client::builder()
            .user_agent(APP_USER_AGENT)
            .hickory_dns(true)
            .build()?;

        let store_path = get_store_path()?;
        let builder = matrix_sdk::Client::builder()
            .http_client(http.clone())
            .request_config(RequestConfig::new().timeout(APP_DEFAULT_TIMEOUT))
            .homeserver_url(&config.matrix.homeserver)
            .sqlite_store(store_path, None);
        let media_proxy = self.media_proxy(&http, &config, &session)?;

        Ok(crate::Client {
            args,
            config,
            session,
            http,
            builder,
            media_proxy,
            with_key_backups: self.with_key_backups,
        })
    }

    fn media_proxy(
        &self,
        http: &reqwest::Client,
        config: &Config,
        session: &MatrixSession,
    ) -> anyhow::Result<Option<Arc<MediaProxy>>> {
        if !self.with_optional_media_proxy {
            return Ok(None);
        }

        match &config.media_proxy {
            Some(_) => {
                use anyhow::Context;

                let jwk = get_jwk_token().context("Locate JWK file failed")?;
                let media_proxy = MediaProxy::new(
                    config.matrix.homeserver.clone(),
                    session.tokens.access_token.clone(),
                    jwk,
                    http,
                )?;
                Ok(Some(Arc::new(media_proxy)))
            }
            None => Ok(None),
        }
    }
}

pub struct Client {
    args: Args,
    config: Config,
    session: MatrixSession,
    http: reqwest::Client,
    builder: matrix_sdk::ClientBuilder,
    media_proxy: Option<Arc<MediaProxy>>,
    with_key_backups: bool,
}

impl Client {
    pub async fn run(self) -> anyhow::Result<()> {
        let Self {
            args,
            config,
            session,
            http,
            builder,
            media_proxy,
            with_key_backups,
        } = self;
        let client = builder.build().await?;
        client.restore_session(session).await?;

        if let Some(command) = args.command {
            match command {
                Subcommand::BootstrapCrossSigning { if_needed } => {
                    if if_needed {
                        crate::matrix::bootstrap_cross_signing_if_needed(&client).await?;
                    } else {
                        crate::matrix::bootstrap_cross_signing(&client).await?;
                    }
                }
                Subcommand::ResetCrossSigning => {
                    crate::matrix::reset_cross_signing(&client).await?;
                }
                Subcommand::RecoverCrossSigning => {
                    crate::matrix::recover_cross_signing(&client).await?;
                }
                Subcommand::CreateSecretStore => {
                    crate::matrix::create_secret_store(&client).await?;
                }
                Subcommand::NewBackup => {
                    crate::matrix::new_backup(&client).await?;
                }
            }

            return Ok(());
        }

        if with_key_backups {
            crate::matrix::enable_key_backups(&client).await?;
        }

        if let Some(ref media_proxy_config) = config.media_proxy {
            crate::start_media_proxy(media_proxy.as_deref(), media_proxy_config.listen.clone());
        }

        let pixiv = pixiv_client(&http, &config).await?;

        let prefix = config.command.prefix.clone();
        let (_, config) = tokio::sync::watch::channel(config);

        let injected = self::message::Injected {
            config,
            prefix,
            http,
            pixiv,
            media_proxy,
        };

        client.add_event_handler_context(injected);
        crate::matrix::log_encryption_info(&client).await?;
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
}
async fn pixiv_client(
    http: &reqwest::Client,
    config: &Config,
) -> anyhow::Result<Option<Arc<PixivClient>>> {
    let Some(ref token) = config.pixiv.token else {
        return Ok(None);
    };
    Ok(Some(Arc::new(PixivClient::from_client(token, http).await?)))
}

fn start_media_proxy(media_proxy: Option<&MediaProxy>, addr: String) {
    let Some(media_proxy) = media_proxy else {
        return;
    };
    let router = media_proxy.router();
    tokio::spawn(
        async move {
            let Ok(listener) = tokio::net::TcpListener::bind(addr).await else {
                return;
            };
            if let Err(e) = axum::serve(listener, router).await {
                tracing::warn!("{e}");
            }
        }
        .instrument(tracing::info_span!("media_proxy")),
    );
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

async fn sync(client: &matrix_sdk::Client) -> anyhow::Result<()> {
    crate::matrix::initial_sync(client).await?;
    if let Err(e) = crate::matrix::ensure_self_device_verified(client).await {
        tracing::warn!("Failed to ensure this device is verified: {e:#}");
    }
    let settings = SyncSettings::default().set_presence(PresenceState::Online);
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

pub(crate) fn imageinfo(data: &Vec<u8>) -> anyhow::Result<ImageInfo> {
    use file_format::FileFormat;
    use matrix_sdk::ruma::UInt;
    use matrix_sdk::ruma::events::room::ThumbnailInfo;

    let dimensions = imagesize::blob_size(data)?;
    let (width, height) = (dimensions.width, dimensions.height);
    let format = FileFormat::from_bytes(data);
    let mimetype = format.media_type();
    let size = data.len();
    let mut thumb = ThumbnailInfo::new();
    let width = UInt::try_from(width)?;
    let height = UInt::try_from(height)?;
    let size = UInt::try_from(size)?;
    thumb.width = Some(width);
    thumb.height = Some(height);
    thumb.mimetype = Some(mimetype.to_string());
    thumb.size = Some(size);
    let mut info = ImageInfo::new();
    info.width = Some(width);
    info.height = Some(height);
    info.mimetype = Some(mimetype.to_string());
    info.size = Some(size);
    info.thumbnail_info = Some(Box::new(thumb));

    Ok(info)
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

    tokio::spawn(
        async move {
            let room_id = room.room_id();
            tracing::info!("Autojoining room {}", room_id);
            let mut delay = 2;
            while let Err(e) = room.join().await {
                use tokio::time::{Duration, sleep};
                tracing::warn!(
                    %room_id,
                    "Failed to join room {room_id} ({e:#}), retrying in {delay}s",
                );
                sleep(Duration::from_secs(delay)).await;
                delay *= 2;

                if delay > 3600 {
                    tracing::error!(
                        %room_id,
                        "Can't join room {room_id} ({e:#})"
                    );
                    break;
                }
            }
        }
        .instrument(tracing::info_span!("on_stripped_member")),
    );
}

/// Called when we have a tombstone event.
#[tracing::instrument(
    skip_all,
    fields(
        room_id = %room.room_id(),
    ),
)]
pub async fn on_room_replace(
    ev: OriginalSyncRoomTombstoneEvent,
    room: matrix_sdk::Room,
    client: matrix_sdk::Client,
) {
    tokio::spawn(async move {
        let room_id: OwnedRoomOrAliasId = ev.content.replacement_room.into();
        tracing::info!(
            room_id = %room.room_id(),
            "Room replaced, Autojoining new room {}",
            room_id
        );
        let sender = ev.sender;
        let server_name = sender.server_name();
        let mut delay = 2;
        while let Err(e) = client
            .join_room_by_id_or_alias(&room_id, &[server_name.into()])
            .await
        {
            use tokio::time::{Duration, sleep};
            tracing::warn!(
                %room_id,
                "Failed to join replacement room {room_id} ({e:#}), retrying in {delay}s"
            );
            sleep(Duration::from_secs(delay)).await;
            delay *= 2;

            if delay > 3600 {
                tracing::error!(
                    %room_id,
                    "Can't join replacement room {room_id} ({e:#}), please join manually."
                );
                break;
            }
        }
        tokio::spawn(async move {
            if let Err(e) = room.leave().await {
                tracing::error!(
                    room_id = %room.room_id(),
                    "Can't leave the original room {} ({e:#})",
                    room.room_id()
                );
            }
        });
    });
}
