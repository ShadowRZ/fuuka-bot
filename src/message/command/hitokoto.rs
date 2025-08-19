use crate::message::Injected;
use matrix_sdk::{
    Room,
    event_handler::Ctx,
    ruma::events::room::message::{AddMentions, ForwardThread, OriginalRoomMessageEvent},
};

pub async fn process(
    ev: &OriginalRoomMessageEvent,
    room: &Room,
    injected: &Ctx<Injected>,
) -> anyhow::Result<()> {
    if let Some(hitokoto) = {
        injected
            .config
            .borrow()
            .services
            .as_ref()
            .and_then(|s| s.hitokoto.clone())
    } {
        let resp = crate::services::hitokoto::request(&injected.http, hitokoto).await?;
        let content = crate::services::hitokoto::format(resp);
        room.send(content.make_reply_to(ev, ForwardThread::No, AddMentions::Yes))
            .await?;
    }

    Ok(())
}
