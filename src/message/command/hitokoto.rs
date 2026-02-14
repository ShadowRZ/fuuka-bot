use crate::Context;
use matrix_sdk::{
    Room,
    event_handler::Ctx,
    ruma::events::room::message::{AddMentions, ForwardThread, OriginalRoomMessageEvent},
};

#[tracing::instrument(name = "hitokoto", skip(ev, room, context), fields(hitokoto_api), err)]
pub async fn process(
    ev: &OriginalRoomMessageEvent,
    room: &Room,
    context: &Ctx<Context>,
) -> anyhow::Result<()> {
    let hitokoto = context.hitokoto.base_url.clone();
    let resp = crate::services::hitokoto::request(&context.http, hitokoto).await?;
    let content = crate::services::hitokoto::format(resp);
    room.send(content.make_reply_to(ev, ForwardThread::No, AddMentions::Yes))
        .await?;

    Ok(())
}
