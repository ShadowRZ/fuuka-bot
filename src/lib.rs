pub mod bot_commands;
pub mod callbacks;
pub mod member_updates;

use callbacks::FuukaBotCallbacks;
use matrix_sdk::{config::SyncSettings, Client, Session};
use serde::{Deserialize, Serialize};
use tracing::{event, Level};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FuukaBotConfig {
    pub command_prefix: String,
    pub homeserver_url: String,
}

#[derive(Clone)]
pub struct FuukaBotContext {
    config: FuukaBotConfig,
}

pub struct FuukaBot {
    client: matrix_sdk::Client,
    context: FuukaBotContext,
}

impl FuukaBot {
    pub async fn new(config: FuukaBotConfig, session: Session) -> anyhow::Result<Self> {
        let builder = Client::builder()
            .homeserver_url(&config.homeserver_url)
            .sled_store("store", None)?;
        let client = builder.build().await?;
        client.restore_login(session).await?;
        let context = FuukaBotContext { config };
        Ok(FuukaBot { client, context })
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        self.client.add_event_handler_context(self.context.clone());
        event!(Level::INFO, "Initial sync beginning...");
        self.client.sync_once(SyncSettings::default()).await?;
        event!(Level::INFO, "Initial sync completed.");
        self.client
            .add_event_handler(FuukaBotCallbacks::on_room_message);
        let settings = SyncSettings::default().token(self.client.sync_token().await.unwrap());
        self.client.sync(settings).await?;
        Ok(())
    }
}
