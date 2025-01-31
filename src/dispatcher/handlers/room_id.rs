use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;
use matrix_sdk::ruma::events::room::message::{AddMentions, ForwardThread};

use super::RequestType;

pub fn event_handler() -> super::EventHandler {
    dptree::case![RequestType::RoomId].endpoint(|request: super::IncomingRequest| async move {
        let super::IncomingRequest { ev, room } = request;

        let room_id: String = room.room_id().into();
        room.send(RoomMessageEventContent::text_plain(room_id).make_reply_to(
            &ev,
            ForwardThread::No,
            AddMentions::Yes,
        ))
        .await?;

        Ok(())
    })
}
