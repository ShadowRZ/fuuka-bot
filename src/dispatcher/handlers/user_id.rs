use std::sync::Arc;

use matrix_sdk::ruma::events::room::message::OriginalRoomMessageEvent;
use matrix_sdk::ruma::events::room::message::{AddMentions, ForwardThread};
use matrix_sdk::ruma::events::{
    room::message::RoomMessageEventContent, AnyMessageLikeEventContent,
};
use matrix_sdk::Room;

use crate::RoomExt;

use super::{Event, OutgoingContent, OutgoingResponse};

pub fn event_handler() -> super::EventHandler {
    dptree::case![Event::UserId].endpoint(
        |ev: Arc<OriginalRoomMessageEvent>, room: Arc<Room>| async move {
            let user_id = room.in_reply_to_target_fallback(&ev).await?;

            Ok(OutgoingResponse {
                room,
                content: OutgoingContent::Event(AnyMessageLikeEventContent::RoomMessage(
                    RoomMessageEventContent::text_plain(user_id).make_reply_to(
                        &ev,
                        ForwardThread::No,
                        AddMentions::Yes,
                    ),
                )),
            })
        },
    )
}
