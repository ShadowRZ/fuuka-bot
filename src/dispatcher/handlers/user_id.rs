use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;
use matrix_sdk::ruma::events::room::message::{AddMentions, ForwardThread};

use crate::RoomExt as _;

use super::RequestType;

pub fn event_handler() -> super::EventHandler {
    dptree::case![RequestType::UserId].endpoint(|request: super::IncomingRequest| async move {
        let super::IncomingRequest { ev, room } = request;

        let user_id = room.in_reply_to_target_fallback(&ev).await?;
        room.send(RoomMessageEventContent::text_plain(user_id).make_reply_to(
            &ev,
            ForwardThread::No,
            AddMentions::Yes,
        ))
        .await?;

        Ok(())
    })
}
