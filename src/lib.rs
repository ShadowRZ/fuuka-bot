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
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
pub mod command;
pub mod config;
pub mod events;
pub mod handler;
pub mod jerryxiao;
pub mod message;
pub mod nahida;
pub mod quote;
#[doc(hidden)]
pub mod session;
pub mod stream;
pub mod types;

pub use crate::config::Config;
pub use crate::handler::Context;
pub use crate::stream::StreamFactory;

use matrix_sdk::matrix_auth::MatrixSession;
use matrix_sdk::room::RoomMember;
use matrix_sdk::ruma::presence::PresenceState;
use matrix_sdk::ruma::MxcUri;
use matrix_sdk::{config::SyncSettings, Client};
use std::sync::Arc;
use thiserror::Error;
use tokio::signal;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use url::Url;

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
            .homeserver_url(&config.homeserver_url)
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
        self.client.add_event_handler_context(http.clone());
        self.client.add_event_handler_context(self.config.clone());
        let task: JoinHandle<()> = tokio::spawn(async move {
            tokio::select! {
                _ = async {
                    let mut initial = true;
                    while let Err(e) = self.sync(initial).await {
                        use tokio::time::{sleep, Duration};
                        tracing::error!("Unexpected error happened, retrying in 10s: {e:#}");
                        sleep(Duration::from_secs(10)).await;
                        initial = false;
                    }
                } => {},
                _ = self.cts.cancelled() => {
                    tracing::info!("Bye!");
                },
            }
        });

        Ok(task.await?)
    }

    async fn sync(&self, initial: bool) -> anyhow::Result<()> {
        let next_batch = self.initial_sync(initial).await?;
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
        self.client.sync(settings).await?;
        Ok(())
    }

    async fn initial_sync(&self, register_handler: bool) -> anyhow::Result<String> {
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

        if register_handler {
            self.client
                .add_event_handler(crate::handler::on_sync_message);
            self.client
                .add_event_handler(crate::handler::on_stripped_member);
            self.client
                .add_event_handler(crate::handler::on_room_replace);
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

/// Extensions to [RoomMember].
pub trait RoomMemberExt {
    /// Returns the display name or the user ID of the specified [RoomMember].
    fn name_or_id(&self) -> &str;
    /// Constructs a HTML link of the specified [RoomMember], known as the mention "pill".
    fn make_pill(&self) -> String;
}

impl RoomMemberExt for RoomMember {
    fn name_or_id(&self) -> &str {
        self.display_name().unwrap_or(self.user_id().as_str())
    }

    fn make_pill(&self) -> String {
        format!(
            "<a href=\"{}\">@{}</a>",
            self.user_id().matrix_to_uri(),
            self.name()
        )
    }
}

/// Extensions to [MxcUri].
pub trait MxcUriExt {
    /// Returns the HTTP URL of the given [MxcUri], with the specified homeserver
    /// using the [Client-Server API](https://spec.matrix.org/latest/client-server-api/#get_matrixmediav3downloadservernamemediaid).
    fn http_url(&self, homeserver: &Url) -> anyhow::Result<Url>;
}

impl MxcUriExt for MxcUri {
    #[tracing::instrument(err)]
    fn http_url(&self, homeserver: &Url) -> anyhow::Result<Url> {
        let (server_name, media_id) = self.parts()?;
        Ok(homeserver
            .join("/_matrix/media/r0/download/")?
            .join(format!("{}/{}", server_name, media_id).as_str())?)
    }
}

/// Error types.
#[derive(Error, Debug)]
pub enum Error {
    /// This command requires replying to an event.
    #[error("Replying to a event is required for this command")]
    RequiresReply,
    /// This command is missing an argument.
    #[error("Missing an argument: {0}")]
    MissingArgument(&'static str),
    /// Invaild argument passed into an argument.
    #[error("Invaild argument passed for {arg}: {source}")]
    InvaildArgument {
        /// The argument that is invaild.
        arg: &'static str,
        #[source]
        /// The source error that caused it to happen.
        source: anyhow::Error,
    },
    /// An unexpected error happened.
    #[error("{0}")]
    UnexpectedError(&'static str),
    /// An unknown command was passed.
    #[error("Unrecognized command {0}")]
    UnknownCommand(String),
}
