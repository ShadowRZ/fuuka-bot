use std::collections::BTreeSet;

use anyhow::Context as _;

use crate::Context;
use matrix_sdk::{
    Room,
    event_handler::Ctx,
    ruma::events::room::message::{AddMentions, ForwardThread, OriginalRoomMessageEvent},
};

#[tracing::instrument(name = "hitokoto", skip(ev, room, context), fields(fuuka_bot.hitokoto.base_url), err)]
pub async fn process(
    ev: &OriginalRoomMessageEvent,
    room: &Room,
    context: &Ctx<Context>,
) -> anyhow::Result<()> {
    let Ctx(Context { hitokoto, .. }) = context;
    let resp = hitokoto
        .request(BTreeSet::new())
        .await
        .context("Failed to request hitokoto")?;
    let content = crate::services::hitokoto::format(resp);
    room.send(content.make_reply_to(ev, ForwardThread::No, AddMentions::Yes))
        .await?;

    Ok(())
}
