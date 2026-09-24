use anyhow::Context as _;
use matrix_sdk::{
    Room,
    event_handler::Ctx,
    ruma::{
        OwnedUserId,
        events::{
            Mentions,
            room::message::{
                AddMentions, ForwardThread, OriginalRoomMessageEvent, RoomMessageEventContent,
            },
        },
    },
};

use crate::{Context, RoomExt as _, RoomMemberExt as _};

#[tracing::instrument(name = "at", skip(ev, room, context), err)]
pub async fn process(
    ev: &OriginalRoomMessageEvent,
    room: &Room,
    context: &Ctx<Context>,
    user_id: OwnedUserId,
) -> anyhow::Result<()> {
    let _ = context;

    let Some(reply_target) = room.in_reply_to_event(ev).await? else {
        room.send_requires_reply().await?;
        return Ok(());
    };

    let Some(member) = room.get_member(&ev.sender).await? else {
        return Ok(());
    };

    let text = format!("@{}", member.name_or_id());
    let html = member.make_pill();

    let url = room
        .matrix_to_event_permalink(reply_target.event_id())
        .await
        .context("Failed to create matrix.to permalink")?;

    room.send(
        RoomMessageEventContent::text_html(
            format!("{} wants your attention on {}", text, url),
            format!("{} wants your attention on {}", html, url),
        )
        .make_reply_to(ev, ForwardThread::No, AddMentions::Yes)
        .add_mentions(Mentions::with_user_ids([user_id])),
    )
    .await?;

    Ok(())
}
