use crate::Context;
use matrix_sdk::{
    Room,
    event_handler::Ctx,
    ruma::{
        OwnedUserId,
        events::room::message::{
            AddMentions, ForwardThread, OriginalRoomMessageEvent, RoomMessageEventContent,
        },
    },
};

#[tracing::instrument(name = "unignore", skip(ev, room, context), err)]
pub async fn process(
    ev: &OriginalRoomMessageEvent,
    room: &Room,
    context: &Ctx<Context>,
    user_id: OwnedUserId,
) -> anyhow::Result<()> {
    let _ = context;

    {
        let sender = &ev.sender;
        let admin = context.admin_user.as_ref();

        if admin != Some(sender) {
            return Ok(());
        };
    }

    let account = room.client().account();
    account.unignore_user(&user_id).await?;

    room.send(RoomMessageEventContent::text_plain("Done.").make_reply_to(
        ev,
        ForwardThread::No,
        AddMentions::Yes,
    ))
    .await?;

    Ok(())
}
