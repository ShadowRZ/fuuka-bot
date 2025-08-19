use crate::message::Injected;
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
pub async fn process(
    ev: &OriginalRoomMessageEvent,
    room: &Room,
    injected: &Ctx<Injected>,
    user_id: OwnedUserId,
) -> anyhow::Result<()> {
    let _ = injected;

    {
        let sender = &ev.sender;
        let config = injected.config.borrow();
        let admin = config.admin_user.as_ref();

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
