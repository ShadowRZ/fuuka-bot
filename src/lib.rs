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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FuukaBotConfig {
    pub command_prefix: String,
    pub homeserver_url: String,
    #[serde(default)]
    pub features: HashMap<String, FuukaBotFeatures>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FuukaBotFeatures {
    #[serde(default)]
    pub jerryxiao: bool,
    #[serde(default)]
    pub randomdraw: bool,
}

pub struct FuukaBotContext {
    config: FuukaBotConfig,
}

pub struct FuukaBot {
    client: matrix_sdk::Client,
    context: Arc<FuukaBotContext>,
}

impl FuukaBot {
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

    pub async fn run(&self) -> anyhow::Result<()> {
        self.client.add_event_handler_context(self.context.clone());
        tracing::info!("Initial sync beginning...");
        let response = self.client.sync_once(SyncSettings::default()).await?;
        tracing::info!("Initial sync completed.");
        self.client
            .add_event_handler(FuukaBotCallbacks::on_room_message);
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

#[derive(Debug, Error)]
pub enum FuukaBotError {
    #[error("To run this command, the sending user should be able to ban users (on the Matrix side, if applies).")]
    RequiresBannable,
    #[error("This command should be used on a reply.")]
    RequiresReply,
    #[error("Command is missing required paramter: {0}.")]
    MissingParamter(&'static str),
    #[error("The specified user does not exist.")]
    UserNotFound,
    #[error("This user should be avaliable.")]
    ShouldAvaliable,
}
