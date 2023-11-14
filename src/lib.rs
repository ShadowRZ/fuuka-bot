//! Fuuka Bot Internals for interested.
//!
//! **WARNING: External crate links are broken in the build documentation on GitHub Pages, sorry.**
#[warn(missing_docs)]
#[warn(rustdoc::missing_crate_level_docs)]
pub mod bot_commands;
pub mod callbacks;
pub mod dicer;
pub mod jerryxiao;
pub mod member_updates;
pub mod message_responses;
pub mod utils;

use callbacks::FuukaBotCallbacks;
use matrix_sdk::matrix_auth::MatrixSession;
use matrix_sdk::ruma::RoomId;
use matrix_sdk::{config::SyncSettings, Client};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;

use crate::message_responses::FuukaBotMessages;

/// The config of Fuuka bot.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FuukaBotConfig {
    /// Command prefix.
    pub command_prefix: String,
    /// The homeserver URL to connect to.
    pub homeserver_url: String,
    /// Optional room features.
    #[serde(default)]
    pub features: HashMap<String, FuukaBotFeatures>,
}

/// What message features are avaliable.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FuukaBotFeatures {
    /// Enable Jerry Xiao like functions.
    #[serde(default)]
    pub jerryxiao: bool,
    /// Enable randomdraw.
    #[serde(default)]
    pub randomdraw: bool,
}

/// Global context data for handlers.
pub struct FuukaBotContext {
    /// The config of Fuuka bot.
    config: FuukaBotConfig,
}

/// The bot itself.
pub struct FuukaBot {
    client: matrix_sdk::Client,
    context: Arc<FuukaBotContext>,
}

impl FuukaBot {
    /// Constructs the bot instance using the given `config` and `session`.
    pub async fn new(config: FuukaBotConfig, session: MatrixSession) -> anyhow::Result<Self> {
        let builder = Client::builder()
            .homeserver_url(&config.homeserver_url)
            .sqlite_store("store", None);
        let client = builder.build().await?;
        client.restore_session(session).await?;
        let context = FuukaBotContext { config };
        Ok(FuukaBot {
            client,
            context: context.into(),
        })
    }

    /// Run this bot.
    pub async fn run(&self) -> anyhow::Result<()> {
        self.client.add_event_handler_context(self.context.clone());
        tracing::info!("Initial sync beginning...");
        let response = self.client.sync_once(SyncSettings::default()).await?;
        tracing::info!("Initial sync completed.");
        self.client
            .add_event_handler(FuukaBotCallbacks::on_room_command);
        self.client.add_event_handler(FuukaBotMessages::dicer);
        // Register room specific handlers.
        for (room, feature) in &self.context.config.features {
            let room = <&RoomId>::try_from(room.as_str())?;
            if feature.jerryxiao {
                self.client
                    .add_room_event_handler(room, FuukaBotMessages::jerryxiao);
            }
            if feature.randomdraw {
                self.client
                    .add_room_event_handler(room, FuukaBotMessages::randomdraw);
            }
        }
        let settings = SyncSettings::default().token(response.next_batch);
        self.client.sync(settings).await?;

        Ok(())
    }
}

/// Error types.
#[derive(Debug, Error)]
pub enum FuukaBotError {
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
    // Internal errors.
    /// The bot encountered an internal error that the user it checked should be avaliable but didn't.
    #[error("This user should be avaliable.")]
    ShouldAvaliable,
}
