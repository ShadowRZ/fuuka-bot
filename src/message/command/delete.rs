use crate::{Context, RoomExt};
use matrix_sdk::{Room, event_handler::Ctx, ruma::events::room::message::OriginalRoomMessageEvent};

#[tracing::instrument(name = "help", skip(ev, room, context), err)]
pub async fn process(
    ev: &OriginalRoomMessageEvent,
    room: &Room,
    context: &Ctx<Context>,
) -> anyhow::Result<()> {
    let _ = context;

    let ev = room
        .in_reply_to_event(ev)
        .await?
        .ok_or(crate::Error::RequiresReply)?;

    if Some(ev.sender()) == room.client().user_id() {
        room.redact(ev.event_id(), None, None).await?;
    }

    Ok(())
}
