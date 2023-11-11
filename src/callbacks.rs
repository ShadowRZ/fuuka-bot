use crate::bot_commands::fuuka_bot_dispatch_command;
use crate::FuukaBotContext;
use anyhow::Error;
use matrix_sdk::event_handler::Ctx;
use matrix_sdk::room::{Joined, Room};
use matrix_sdk::ruma::events::room::message::sanitize::remove_plain_reply_fallback;
use matrix_sdk::ruma::events::room::message::OriginalSyncRoomMessageEvent;
use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;
use std::sync::Arc;
pub struct FuukaBotCallbacks;

impl FuukaBotCallbacks {
    pub async fn on_room_message(
        ev: OriginalSyncRoomMessageEvent,
        room: Room,
        ctx: Ctx<Arc<FuukaBotContext>>,
    ) -> anyhow::Result<()> {
        let client = room.client();
        let user_id = client.user_id().unwrap();
        if ev.sender == user_id {
            return Ok(());
        }
        if let Room::Joined(room) = room {
            let body = remove_plain_reply_fallback(ev.content.body()).trim();
            if let Some(commands) = body.strip_prefix(&ctx.config.command_prefix) {
                if let Err(e) = fuuka_bot_dispatch_command(ev.clone(), room.clone(), commands).await
                {
                    send_error_message(ev, room, e).await?;
                }
            }
        }

        Ok(())
    }
}

async fn send_error_message(
    ev: OriginalSyncRoomMessageEvent,
    room: Joined,
    err: Error,
) -> anyhow::Result<()> {
    let content = RoomMessageEventContent::text_plain(format!("{:#}", err))
        .make_reply_to(&ev.into_full_event(room.room_id().into()));
    room.send(content, None).await?;

    // Send this error back to log to tracing.
    Err(err)
}
