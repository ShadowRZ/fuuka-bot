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
pub mod command;
pub mod config;
pub mod de;
pub mod events;
pub mod handler;
pub mod message;
pub mod services;
#[doc(hidden)]
pub mod session;
pub mod traits;
pub mod types;

pub use crate::config::Config;
pub use crate::handler::Context;
pub use crate::traits::*;
pub use crate::types::Error;

use matrix_sdk::matrix_auth::MatrixSession;
use matrix_sdk::ruma::presence::PresenceState;
use matrix_sdk::{config::SyncSettings, Client};
use pixrs::PixivClient;
use std::path::PathBuf;
use std::sync::Arc;
use thiserror::Error;
use tokio::signal;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

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
    client: matrix_sdk::Client,
    config: Arc<Config>,
    cts: CancellationToken,
}

impl FuukaBot {
    /// Constructs the bot instance using the given `config` and `session`.
    pub async fn new(config: Config, session: MatrixSession) -> anyhow::Result<Self> {
        let store_path = Self::store_path()?;
        let builder = Client::builder()
            .homeserver_url(&config.matrix.homeserver)
            .sqlite_store(store_path, None);
        let client = builder.build().await?;
        client.restore_session(session).await?;
        let config = config.into();
        let cts = CancellationToken::new();
        Ok(FuukaBot {
            client,
            cts,
            config,
        })
    }

    /// Run this bot.
    #[tracing::instrument(skip_all)]
    pub async fn run(self) -> anyhow::Result<()> {
        let http = reqwest::Client::builder()
            .user_agent(APP_USER_AGENT)
            .build()?;
        let pixiv = self.pixiv_client().await?;
        self.client.add_event_handler_context(http.clone());
        self.client.add_event_handler_context(self.config.clone());
        self.client.add_event_handler_context(pixiv.clone());
        let task: JoinHandle<()> = tokio::spawn(async move {
            tokio::select! {
                _ = async {
                    while let Err(e) = self.sync().await {
                        use tokio::time::{sleep, Duration};
                        tracing::error!("Unexpected error happened, retrying in 10s: {e:#}");
                        sleep(Duration::from_secs(10)).await;
                    }
                } => {},
                _ = self.cts.cancelled() => {
                    tracing::info!("Bye!");
                },
            }
        });

        Ok(task.await?)
    }

    async fn pixiv_client(&self) -> anyhow::Result<Option<Arc<PixivClient>>> {
        let Some(ref token) = self.config.pixiv.token else {
            return Ok(None);
        };
        Ok(Some(Arc::new(PixivClient::new(token).await?)))
    }

    #[tracing::instrument(skip_all)]
    async fn sync(&self) -> anyhow::Result<()> {
        let next_batch = self.initial_sync().await?;
        // Print some info regarding this device.
        {
            let encryption = self.client.encryption();
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
        if let Err(e) = self.ensure_self_device_verified().await {
            tracing::warn!("Failed to ensure this device is verified: {e:#}");
        }
        let settings = SyncSettings::default()
            .token(next_batch)
            .set_presence(PresenceState::Online);
        use matrix_sdk::ruma::api::client::presence::set_presence::v3::Request;
        if let Some(user_id) = self.client.user_id() {
            let mut presence = Request::new(user_id.into(), PresenceState::Online);
            presence.status_msg = Some(APP_PRESENCE_TEXT.to_string());
            if let Err(e) = self.client.send(presence, None).await {
                tracing::warn!("Failed to set presence: {e:#}");
            }
        }

        let h1 = self
            .client
            .add_event_handler(crate::handler::on_sync_message);
        let h2 = self
            .client
            .add_event_handler(crate::handler::on_stripped_member);
        let h3 = self
            .client
            .add_event_handler(crate::handler::on_room_replace);

        if let Err(e) = self.client.sync(settings).await {
            self.client.remove_event_handler(h1);
            self.client.remove_event_handler(h2);
            self.client.remove_event_handler(h3);
            return Err(e.into());
        }
        Ok(())
    }

    async fn ensure_self_device_verified(&self) -> anyhow::Result<()> {
        let encryption = self.client.encryption();
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

    #[tracing::instrument(skip_all)]
    async fn initial_sync(&self) -> anyhow::Result<String> {
        let user_id = self.client.user_id();
        let homeserver = self.client.homeserver();
        tracing::info!(user_id = ?user_id, homeserver = %homeserver, "Initial sync beginning...");
        let response = self
            .client
            .sync_once(SyncSettings::default().set_presence(PresenceState::Online))
            .await?;
        tracing::info!(user_id = ?user_id, homeserver = %homeserver, "Initial sync completed.");
        if let Some(admin_user) = self.config.admin_user.as_ref() {
            if let Some(admin_room) = self.client.get_dm_room(admin_user) {
                tracing::debug!(
                    room_id = %admin_room.room_id(),
                    user_id = %admin_user,
                    "Found a DM room with admin."
                );
            }
        }

        Ok(response.next_batch)
    }

    fn store_path() -> anyhow::Result<PathBuf> {
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

    /// Enable encrypted message recovery.
    #[tracing::instrument(skip_all)]
    pub async fn enable_recovery(self) -> anyhow::Result<Self> {
        let backup = self.client.encryption().backups();

        if backup.are_enabled().await && backup.exists_on_server().await? {
            tracing::debug!(
                "Bot has an existing server key backup that is valid, skipping recovery provision."
            );
            return Ok(self);
        }

        let recovery = self.client.encryption().recovery();
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

        Ok(self)
    }

    /// Registers the graceful shutdown handler.
    pub fn with_shutdown(self) -> Self {
        let cts = self.cts.clone();

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

        self
    }

    /// Prepares the bootstrap cross signing key if needed.
    pub async fn bootstrap_cross_signing_if_needed(&self) -> anyhow::Result<()> {
        use anyhow::Context;
        use matrix_sdk::ruma::api::client::uiaa;
        use rpassword::read_password;

        if let Err(e) = self
            .client
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
                    uiaa::UserIdentifier::UserIdOrLocalpart(
                        self.client.user_id().unwrap().to_string(),
                    ),
                    password,
                );
                password.session = response.session.clone();

                self.client
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
    pub async fn bootstrap_cross_signing(&self) -> anyhow::Result<()> {
        use anyhow::Context;
        use matrix_sdk::ruma::api::client::uiaa;
        use rpassword::read_password;

        if let Err(e) = self.client.encryption().bootstrap_cross_signing(None).await {
            if let Some(response) = e.as_uiaa_response() {
                use std::io::Write;

                print!("Enter password for preparing cross signing: ");
                std::io::stdout().flush()?;
                let password = read_password()?;
                let mut password = uiaa::Password::new(
                    uiaa::UserIdentifier::UserIdOrLocalpart(
                        self.client.user_id().unwrap().to_string(),
                    ),
                    password,
                );
                password.session = response.session.clone();

                self.client
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
    pub async fn reset_cross_signing(&self) -> anyhow::Result<()> {
        use matrix_sdk::encryption::CrossSigningResetAuthType;

        if let Some(handle) = self.client.encryption().reset_cross_signing().await? {
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
                            self.client.user_id().unwrap().to_string(),
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
