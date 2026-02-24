use crate::Context;
use anyhow::Context as _;
use matrix_sdk::{
    Room,
    event_handler::Ctx,
    ruma::events::room::message::{AddMentions, ForwardThread, OriginalRoomMessageEvent},
};

#[tracing::instrument(name = "bilibili", skip(ev, room, context), err)]
pub async fn process(
    ev: &OriginalRoomMessageEvent,
    room: &Room,
    context: &Ctx<Context>,
    id: &str,
) -> anyhow::Result<()> {
    let video = crate::services::bilibili::video::request(&context.http, id)
        .await
        .context(format!("Failed to query BiliBili video {id}"))?;
    let content = crate::services::bilibili::video::format(video, false)?;

    room.send(content.make_reply_to(ev, ForwardThread::No, AddMentions::Yes))
        .await?;

    Ok(())
}
