use std::sync::Arc;

use matrix_sdk::event_handler::EventHandlerHandle;
use matrix_sdk::room::RoomMember;
use matrix_sdk::ruma::events::room::message::{
    AddMentions, ForwardThread, RoomMessageEventContent,
};
use matrix_sdk::ruma::events::AnyMessageLikeEventContent;
use matrix_sdk::{ruma::events::room::message::OriginalRoomMessageEvent, Room};
use matrix_sdk::ruma::events::room::message::OriginalSyncRoomMessageEvent;
use matrix_sdk::ruma::events::Mentions;
use matrix_sdk::ruma::OwnedUserId;

use crate::RoomMemberExt as _;

use super::{Event, OutgoingContent, OutgoingResponse};

pub fn event_handler() -> super::EventHandler {
    dptree::case![Event::Remind {
        target,
        sender,
        content
    }]
    .endpoint(
        |(target, sender, content): (OwnedUserId, Arc<RoomMember>, Option<String>),
         ev: Arc<OriginalRoomMessageEvent>,
         room: Arc<Room>| async move {
            room.add_event_handler(
                |ev: OriginalSyncRoomMessageEvent,
                 client: matrix_sdk::Client,
                 room: Room,
                 handle: EventHandlerHandle| async move {
                    let ev = ev.into_full_event(room.room_id().into());
                    if ev.sender == target {
                        let pill = sender.make_pill();
                        let reminder = content.unwrap_or("You can ask now.".to_string());
                        let content = RoomMessageEventContent::text_html(
                            format!("Cc {} {}", sender.name_or_id(), &reminder),
                            format!("Cc {} {}", pill, &reminder),
                        )
                        .make_reply_to(&ev, ForwardThread::No, AddMentions::Yes)
                        .add_mentions(Mentions::with_user_ids([target]));
                        match room.send(content).await {
                            Ok(_) => (),
                            Err(e) => tracing::error!("Unexpected error happened: {e:#}"),
                        }
                        client.remove_event_handler(handle);
                    };
                },
            );

            Ok(OutgoingResponse {
                room,
                content: OutgoingContent::Event(AnyMessageLikeEventContent::RoomMessage(
                    RoomMessageEventContent::text_plain(
                        "You'll be reminded when the target speaks.",
                    )
                    .make_reply_to(&ev, ForwardThread::No, AddMentions::Yes),
                )),
            })
        },
    )
}
