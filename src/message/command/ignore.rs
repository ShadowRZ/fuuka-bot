use crate::message::Injected;
use matrix_sdk::{
    Room,
    event_handler::Ctx,
    ruma::events::room::message::{
        AddMentions, ForwardThread, OriginalRoomMessageEvent, RoomMessageEventContent,
    },
};

#[tracing::instrument(name = "ignore", skip(ev, room, injected), fields(will_ignore), err)]
pub async fn process(
    ev: &OriginalRoomMessageEvent,
    room: &Room,
    injected: &Ctx<Injected>,
) -> anyhow::Result<()> {
    let _ = injected;
    use crate::RoomExt as _;

    {
        let sender = &ev.sender;
        let config = injected.config.borrow();
        let admin = config.admin_user.as_ref();

        if admin != Some(sender) {
            return Ok(());
        };
    }

    let user_id = room
        .in_reply_to_target(ev)
        .await?
        .ok_or(crate::Error::RequiresReply)?;
    tracing::Span::current().record("will_ignore", tracing::field::display(&user_id));
    let account = room.client().account();
    account.ignore_user(&user_id).await?;

    room.send(RoomMessageEventContent::text_plain("Done.").make_reply_to(
        ev,
        ForwardThread::No,
        AddMentions::Yes,
    ))
    .await?;

    Ok(())
}
