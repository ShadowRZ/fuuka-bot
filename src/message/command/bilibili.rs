use crate::message::Injected;
use matrix_sdk::{
    Room,
    event_handler::Ctx,
    ruma::events::room::message::{AddMentions, ForwardThread, OriginalRoomMessageEvent},
};

#[tracing::instrument(name = "bilibili", skip(ev, room, injected), err)]
pub async fn process(
    ev: &OriginalRoomMessageEvent,
    room: &Room,
    injected: &Ctx<Injected>,
    id: &str,
) -> anyhow::Result<()> {
    let video = crate::services::bilibili::video::request(&injected.http, id).await?;
    let content = crate::services::bilibili::video::format(video, false)?;

    room.send(content.make_reply_to(ev, ForwardThread::No, AddMentions::Yes))
        .await?;

    Ok(())
}
