use std::sync::Arc;

use matrix_sdk::ruma::events::room::message::OriginalRoomMessageEvent;
use matrix_sdk::ruma::events::room::message::{AddMentions, ForwardThread};
use matrix_sdk::ruma::events::{
    room::message::RoomMessageEventContent, AnyMessageLikeEventContent,
};
use matrix_sdk::Room;

use super::{Event, OutgoingContent, OutgoingResponse};

pub fn event_handler() -> super::EventHandler {
    dptree::case![Event::RoomId].endpoint(
        |ev: Arc<OriginalRoomMessageEvent>, room: Arc<Room>| async move {
            let room_id: String = room.room_id().into();
            Ok(OutgoingResponse {
                room,
                content: OutgoingContent::Event(AnyMessageLikeEventContent::RoomMessage(
                    RoomMessageEventContent::text_plain(room_id).make_reply_to(
                        &ev,
                        ForwardThread::No,
                        AddMentions::Yes,
                    ),
                )),
            })
        },
    )
}
