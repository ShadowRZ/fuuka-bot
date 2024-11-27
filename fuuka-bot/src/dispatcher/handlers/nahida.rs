use std::sync::Arc;

use matrix_sdk::ruma::events::{
    room::message::{AddMentions, ForwardThread, OriginalRoomMessageEvent},
    AnyMessageLikeEventContent,
};
use matrix_sdk::Room;
use url::Url;

use super::{Event, OutgoingContent, OutgoingResponse};

pub fn event_handler() -> super::EventHandler {
    dptree::case![Event::Nahida(url,)].endpoint(
        |(url,): (Url,),
         ev: Arc<OriginalRoomMessageEvent>,
         room: Arc<Room>,
         config: Arc<crate::Config>,
         http: reqwest::Client,
         pixiv: Option<Arc<pixrs::PixivClient>>| async move {
            let content =
                crate::message::nahida::dispatch(url, &room, &config, &http, pixiv.as_deref())
                    .await?;

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
    )
}
