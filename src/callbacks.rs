//! Generic Matrix event callback handler.

use crate::bot_commands::fuuka_bot_dispatch_command;
use crate::FuukaBotContext;
use crate::FuukaBotError;
use matrix_sdk::event_handler::Ctx;
use matrix_sdk::room::Room;
use matrix_sdk::ruma::events::room::member::StrippedRoomMemberEvent;
use matrix_sdk::ruma::events::room::message::sanitize::remove_plain_reply_fallback;
use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;
use matrix_sdk::ruma::events::room::message::{
    AddMentions, ForwardThread, OriginalSyncRoomMessageEvent,
};
use matrix_sdk::RoomState;
use std::sync::Arc;

/// A ZST for containing callbacks.
pub struct FuukaBotCallbacks;

impl FuukaBotCallbacks {
    /// The callback handler for commands.
    pub async fn on_room_command(
        ev: OriginalSyncRoomMessageEvent,
        room: Room,
        ctx: Ctx<Arc<FuukaBotContext>>,
    ) -> anyhow::Result<()> {
        // It should be a joined room.
        if room.state() != RoomState::Joined {
            return Ok(());
        }
        let client = room.client();
        let user_id = client.user_id().unwrap();
        if ev.sender == user_id {
            return Ok(());
        }

        let body = remove_plain_reply_fallback(ev.content.body()).trim();
        if let Some(commands) = body.strip_prefix(&ctx.config.command_prefix) {
            if let Err(e) =
                fuuka_bot_dispatch_command(ev.clone(), room.clone(), commands, client.homeserver())
                    .await
            {
                send_error_message(ev, room, e).await?;
            }
        }

        Ok(())
    }

    pub async fn on_stripped_member(ev: StrippedRoomMemberEvent, room: Room) {
        let client = room.client();
        let user_id = client.user_id().unwrap();
        if ev.state_key != user_id {
            return;
        }

        tokio::spawn(async move {
            let room_id = room.room_id();
            tracing::info!("Autojoining room {}", room_id);
            let mut delay = 2;
            while let Err(e) = room.join().await {
                use tokio::time::{sleep, Duration};
                tracing::warn!("Failed to join room {room_id} ({e:?}), retrying in {delay}s");
                sleep(Duration::from_secs(delay)).await;
                delay *= 2;

                if delay > 3600 {
                    tracing::error!("Can't join room {room_id} ({e:?})");
                    break;
                }
            }
        });
    }
}

async fn send_error_message(
    ev: OriginalSyncRoomMessageEvent,
    room: Room,
    err: anyhow::Error,
) -> anyhow::Result<()> {
    let content = match err.downcast_ref::<FuukaBotError>() {
        Some(FuukaBotError::MissingParamter(_)) => {
            RoomMessageEventContent::text_plain(format!("Invaild input: {err:#}"))
        }
        Some(FuukaBotError::RequiresBannable | FuukaBotError::RequiresReply) => {
            RoomMessageEventContent::text_plain(format!(
                "Command requirement is unsatisfied: {err:#}"
            ))
        }
        Some(FuukaBotError::UserNotFound) => {
            RoomMessageEventContent::text_plain(format!("Runtime error: {err:#}"))
        }
        Some(&FuukaBotError::ShouldAvaliable) => RoomMessageEventContent::text_plain(format!(
            "⁉️ The bot fired an internal error: {err:#}"
        )),
        Some(&FuukaBotError::MathOverflow) | Some(&FuukaBotError::DivByZero) => {
            RoomMessageEventContent::text_plain(format!("⁉️ Math error happened: {err:#}"))
        }
        None => {
            RoomMessageEventContent::text_plain(format!("⁉️ An unexpected error occoured: {err:#}"))
        }
    };
    let content = content.make_reply_to(
        &ev.into_full_event(room.room_id().into()),
        ForwardThread::Yes,
        AddMentions::Yes,
    );
    room.send(content).await?;

    // Send this error back to log to tracing.
    Err(err)
}
