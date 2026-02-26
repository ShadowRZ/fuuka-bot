use crate::Context;
use crate::RoomExt as _;
use matrix_sdk::{
    Room,
    event_handler::Ctx,
    ruma::events::room::message::{
        AddMentions, ForwardThread, OriginalRoomMessageEvent, RoomMessageEventContent,
    },
};

#[tracing::instrument(name = "user_id", skip_all, err)]
pub async fn process(
    ev: &OriginalRoomMessageEvent,
    room: &Room,
    context: &Ctx<Context>,
) -> anyhow::Result<()> {
    let _ = context;

    let user_id = room.in_reply_to_target_fallback(ev).await?;
    room.send(RoomMessageEventContent::text_plain(user_id).make_reply_to(
        ev,
        ForwardThread::No,
        AddMentions::Yes,
    ))
    .await?;

    Ok(())
}
