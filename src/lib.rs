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
pub mod dicer;
pub mod handler;
pub mod jerryxiao;
pub mod message;
pub mod nahida;
pub mod quote;
#[doc(hidden)]
pub mod session;
pub mod stream;
pub mod traits;
pub mod types;

pub use crate::stream::StreamFactory;
pub use crate::traits::{IntoEventContent, MxcUriExt, RoomMemberExt};

use matrix_sdk::matrix_auth::MatrixSession;
use matrix_sdk::ruma::events::room::message::sanitize::remove_plain_reply_fallback;
use matrix_sdk::ruma::events::room::message::OriginalRoomMessageEvent;
use matrix_sdk::ruma::events::room::message::OriginalSyncRoomMessageEvent;
use matrix_sdk::ruma::events::room::message::Relation;
use matrix_sdk::ruma::events::AnyTimelineEvent;
use matrix_sdk::ruma::presence::PresenceState;
use matrix_sdk::ruma::{OwnedRoomId, OwnedUserId};
use matrix_sdk::Room;
use matrix_sdk::{config::SyncSettings, Client};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
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

/// The config of Fuuka bot.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    /// Command prefix.
    pub command_prefix: String,
    /// The homeserver URL to connect to.
    pub homeserver_url: Url,
    /// Optional room features.
    #[serde(default)]
    pub features: HashMap<OwnedRoomId, RoomFeatures>,
    /// HTTP Services configuration.
    pub services: ServiceBackends,
}

/// What message features are avaliable.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RoomFeatures {
    /// Enable Jerry Xiao like functions.
    #[serde(default)]
    pub jerryxiao: bool,
    /// Enable randomdraw.
    #[serde(default)]
    pub randomdraw: bool,
}

/// Configure various backend APIs
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServiceBackends {
    /// Hitokoto API endpoint.
    /// The API should implment <https://developer.hitokoto.cn/sentence/#%E6%8E%A5%E5%8F%A3%E8%AF%B4%E6%98%8E>.
    pub hitokoto: Url,
}

/// Global context data for handlers.
pub struct BotContext {
    /// The config of Fuuka bot.
    config: Config,
    /// HTTP client used for HTTP APIs.
    http_client: reqwest::Client,
}

/// The bot itself.
pub struct FuukaBot {
    client: matrix_sdk::Client,
    context: Arc<BotContext>,
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
        let http_client = reqwest::Client::builder()
            .user_agent(APP_USER_AGENT)
            .build()?;
        let context = BotContext {
            config,
            http_client,
        };
        Ok(FuukaBot {
            client,
            context: context.into(),
            cts: CancellationToken::new(),
        })
    }

    /// Run this bot.
    pub async fn run(self) -> anyhow::Result<()> {
        self.client.add_event_handler_context(self.context.clone());
        let task: JoinHandle<()> = tokio::spawn(async move {
            tokio::select! {
                _ = async {
                    let mut initial = true;
                    while let Err(e) = self.sync(initial).await {
                        use tokio::time::{sleep, Duration};
                        tracing::error!("Unexpected error happened, retrying in 10s: {e:?}");
                        sleep(Duration::from_secs(10)).await;
                        initial = false;
                    }
                } => {},
                _ = self.cts.cancelled() => {},
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
        tracing::info!("Initial sync beginning...");
        let response = self
            .client
            .sync_once(SyncSettings::default().set_presence(PresenceState::Online))
            .await?;
        tracing::info!("Initial sync completed.");

        if register_handler {
            self.client
                .add_event_handler(crate::handler::on_sync_message);
            self.client
                .add_event_handler(crate::handler::on_stripped_member);
        }

        Ok(response.next_batch)
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

/// Error types.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Running this command requires the sending user should be able to ban users (on the Matrix side, if applies).
    #[error("To run this command, the sending user should be able to ban users (on the Matrix side, if applies).")]
    RequiresBannable,
    /// The command should be used on a reply.
    #[error("This command should be used on a reply.")]
    RequiresReply,
    /// Command is missing required paramter.
    #[error("Command is missing required paramter: {0}.")]
    MissingParamter(&'static str),
    /// The specified user does not exist.
    #[error("The specified user does not exist.")]
    UserNotFound,
    /// Math overflow happened.
    #[error("Math overflow happened.")]
    MathOverflow,
    /// Divide by zero happened.
    #[error("Divisioned by zero.")]
    DivByZero,
    /// Invaild URL given.
    #[error("Invaild URL given: {0}.")]
    InvaildUrl(#[from] url::ParseError),
    /// No vaild infomation can be extracted.
    #[error("No infomation can be extracted.")]
    NoInfomation,
    // Internal errors.
    /// The bot encountered an internal error that the user it checked should be avaliable but didn't.
    #[error("This user should be avaliable.")]
    ShouldAvaliable,
}

/// Context for the handler.
pub struct HandlerContext {
    /// The event that bot was received.
    pub ev: OriginalRoomMessageEvent,
    /// The room where the event was sent from.
    pub room: Room,
    /// The sender.
    pub sender: OwnedUserId,
    /// The text part of the event content.
    pub body: String,
    /// The homeserver URL.
    pub homeserver: Url,
}

impl HandlerContext {
    /// Creates a context from the given [OriginalSyncRoomMessageEvent], [Room] and [Url].
    pub fn new(ev: OriginalSyncRoomMessageEvent, room: Room, homeserver: Url) -> Self {
        Self {
            ev: ev.clone().into_full_event(room.room_id().into()),
            room,
            sender: ev.sender,
            body: remove_plain_reply_fallback(ev.content.body())
                .trim()
                .to_string(),
            homeserver,
        }
    }
}

// The rest are functions that can't be organized clearly.

/// Given a [OriginalRoomMessageEvent], returns the user ID of the reply target.
pub(crate) async fn get_reply_target(
    ev: &OriginalRoomMessageEvent,
    room: &Room,
) -> anyhow::Result<Option<OwnedUserId>> {
    match &ev.content.relates_to {
        Some(Relation::Reply { in_reply_to }) => {
            let event_id = &in_reply_to.event_id;
            let event = room.event(event_id).await?.event.deserialize()?;
            let ret = event.sender();
            Ok(Some(ret.into()))
        }
        _ => Ok(None),
    }
}

/// Given a [OriginalRoomMessageEvent], returns the event being replied to.
pub(crate) async fn get_reply_event(
    ev: &OriginalRoomMessageEvent,
    room: &Room,
) -> anyhow::Result<Option<AnyTimelineEvent>> {
    match &ev.content.relates_to {
        Some(Relation::Reply { in_reply_to }) => {
            let event_id = &in_reply_to.event_id;
            let event = room.event(event_id).await?.event.deserialize()?;
            Ok(Some(event))
        }
        _ => Ok(None),
    }
}

/// Given a [OriginalRoomMessageEvent], returns the user ID of the reply target,
/// it that doesn't exist, returns the user ID of the sender.
pub(crate) async fn get_reply_target_fallback(
    ev: &OriginalRoomMessageEvent,
    room: &Room,
) -> anyhow::Result<OwnedUserId> {
    get_reply_target(ev, room)
        .await
        .map(|r| r.unwrap_or(ev.sender.clone()))
}
