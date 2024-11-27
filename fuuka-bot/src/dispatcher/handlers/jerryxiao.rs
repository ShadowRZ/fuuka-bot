use std::sync::Arc;

use matrix_sdk::{room::RoomMember, Room};
use matrix_sdk::ruma::events::{
    room::message::{AddMentions, ForwardThread, OriginalRoomMessageEvent},
    AnyMessageLikeEventContent,
};

use super::{Event, OutgoingContent, OutgoingResponse};

pub fn event_handler() -> super::EventHandler {
    dptree::entry()
        .branch(dptree::case![Event::Slash { from, to, text }].endpoint(
            |(from, to, text): (Arc<RoomMember>, Arc<RoomMember>, String),
             ev: Arc<OriginalRoomMessageEvent>,
             room: Arc<Room>| async move {
                let content = crate::message::jerryxiao::jerryxiao(&from, &to, &text).await?;

                Ok(OutgoingResponse {
                    room,
                    content: content
                        .map(|content| {
                            OutgoingContent::Event(AnyMessageLikeEventContent::RoomMessage(
                                content.make_reply_to(&ev, ForwardThread::No, AddMentions::Yes),
                            ))
                        })
                        .unwrap_or_default(),
                })
            },
        ))
        .branch(
            dptree::case![Event::SlashFormatted { from, to, text }].endpoint(
                |(from, to, text): (Arc<RoomMember>, Arc<RoomMember>, String),
                 ev: Arc<OriginalRoomMessageEvent>,
                 room: Arc<Room>| async move {
                    let content =
                        crate::message::jerryxiao::jerryxiao_formatted(&from, &to, &text).await?;

                    Ok(OutgoingResponse {
                        room,
                        content: content
                            .map(|content| {
                                OutgoingContent::Event(AnyMessageLikeEventContent::RoomMessage(
                                    content.make_reply_to(&ev, ForwardThread::No, AddMentions::Yes),
                                ))
                            })
                            .unwrap_or_default(),
                    })
                },
            ),
        )
        .branch(
            dptree::case![Event::Fortune { member, text, prob }].endpoint(
                |(member, text, prob): (Arc<RoomMember>, String, bool),
                 ev: Arc<OriginalRoomMessageEvent>,
                 room: Arc<Room>| async move {
                    let content = crate::message::jerryxiao::fortune(&member, &text, prob).await?;

                    Ok(OutgoingResponse {
                        room,
                        content: OutgoingContent::Event(AnyMessageLikeEventContent::RoomMessage(
                            content.make_reply_to(&ev, ForwardThread::No, AddMentions::Yes),
                        )),
                    })
                },
            ),
        )
}
