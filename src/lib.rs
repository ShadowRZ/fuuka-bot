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
        let builder = Client::builder()
            .homeserver_url(&config.matrix.homeserver)
            .sqlite_store("store", None);
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

    async fn sync(&self) -> anyhow::Result<()> {
        let next_batch = self.initial_sync().await?;
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

    /// Disable encrypted message recovery.
    pub async fn disable_recovery(self) -> anyhow::Result<Self> {
        self.client.encryption().recovery().disable().await?;
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
}
