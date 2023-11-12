use crate::bot_commands::fuuka_bot_dispatch_command;
use crate::FuukaBotContext;
use matrix_sdk::event_handler::Ctx;
use matrix_sdk::room::Room;
use matrix_sdk::ruma::events::room::message::sanitize::remove_plain_reply_fallback;
use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;
use matrix_sdk::ruma::events::room::message::{
    AddMentions, ForwardThread, OriginalSyncRoomMessageEvent,
};
use matrix_sdk::RoomState;
use std::sync::Arc;
pub struct FuukaBotCallbacks;

impl FuukaBotCallbacks {
    pub async fn on_room_message(
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
}

async fn send_error_message(
    ev: OriginalSyncRoomMessageEvent,
    room: Room,
    err: anyhow::Error,
) -> anyhow::Result<()> {
    let content =
        RoomMessageEventContent::text_plain(format!("⁉️ An unexpected error occoured: {err:#}"))
            .make_reply_to(
                &ev.into_full_event(room.room_id().into()),
                ForwardThread::Yes,
                AddMentions::Yes,
            );
    room.send(content).await?;

    // Send this error back to log to tracing.
    Err(err)
}
