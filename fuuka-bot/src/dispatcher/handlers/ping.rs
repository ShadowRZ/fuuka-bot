use std::sync::Arc;

use matrix_sdk::ruma::events::room::message::{
    AddMentions, ForwardThread, RoomMessageEventContent,
};
use matrix_sdk::ruma::events::AnyMessageLikeEventContent;
use matrix_sdk::ruma::MilliSecondsSinceUnixEpoch;
use matrix_sdk::{ruma::events::room::message::OriginalRoomMessageEvent, Room};
use time::Duration;

use super::{Event, OutgoingContent, OutgoingResponse};

pub fn event_handler() -> super::EventHandler {
    dptree::case![Event::Ping].endpoint(
        |ev: Arc<OriginalRoomMessageEvent>, room: Arc<Room>| async move {
            let MilliSecondsSinceUnixEpoch(now) = MilliSecondsSinceUnixEpoch::now();
            let MilliSecondsSinceUnixEpoch(event_ts) = ev.origin_server_ts;
            let now = Duration::milliseconds(now.into());
            let event_ts = Duration::milliseconds(event_ts.into());
            let delta = now - event_ts;
            let delta_ms = delta.whole_milliseconds();
            let body = if delta_ms >= 2000 {
                format!("Pong after {delta:.3}")
            } else {
                format!("Pong after {}ms", delta_ms)
            };

            Ok(OutgoingResponse {
                room,
                content: OutgoingContent::Event(AnyMessageLikeEventContent::RoomMessage(
                    RoomMessageEventContent::text_plain(body).make_reply_to(
                        &ev,
                        ForwardThread::No,
                        AddMentions::Yes,
                    ),
                )),
            })
        },
    )
}
