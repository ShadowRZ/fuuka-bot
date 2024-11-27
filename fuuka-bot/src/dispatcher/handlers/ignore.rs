use std::sync::Arc;

use matrix_sdk::ruma::events::room::message::{
    AddMentions, ForwardThread, RoomMessageEventContent,
};
use matrix_sdk::ruma::events::AnyMessageLikeEventContent;
use matrix_sdk::ruma::OwnedUserId;
use matrix_sdk::{ruma::events::room::message::OriginalRoomMessageEvent, Room};

use super::{Event, OutgoingContent, OutgoingResponse};

pub fn event_handler() -> super::EventHandler {
    dptree::case![Event::Ignore(user_id)].endpoint(
        |user_id: OwnedUserId,
         ev: Arc<OriginalRoomMessageEvent>,
         room: Arc<Room>,
         client: matrix_sdk::Client| async move {
            let account = client.account();
            account.ignore_user(&user_id).await?;

            Ok(OutgoingResponse {
                room,
                content: OutgoingContent::Event(AnyMessageLikeEventContent::RoomMessage(
                    RoomMessageEventContent::text_plain("Done.").make_reply_to(
                        &ev,
                        ForwardThread::No,
                        AddMentions::Yes,
                    ),
                )),
            })
        },
    )
}
