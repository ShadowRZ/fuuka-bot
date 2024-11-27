use std::sync::Arc;

use matrix_sdk::ruma::events::room::message::OriginalRoomMessageEvent;
use matrix_sdk::ruma::events::room::message::{AddMentions, ForwardThread};
use matrix_sdk::ruma::events::{
    room::message::RoomMessageEventContent, AnyMessageLikeEventContent,
};
use matrix_sdk::Room;

use crate::types::HitokotoResult;
use crate::Config;

use super::{Event, OutgoingContent, OutgoingResponse};

pub fn event_handler() -> super::EventHandler {
    dptree::case![Event::Hitokoto].endpoint(|
        ev: Arc<OriginalRoomMessageEvent>,
        room: Arc<Room>,
        config: Arc<Config>,
        http: reqwest::Client| async move {
            let content = match config.services.as_ref().and_then(|c| c.hitokoto.as_ref()) {
                Some(hitokoto) => {
                    let raw_resp = http
                        .get(hitokoto.to_owned())
                        .send()
                        .await?
                        .error_for_status()?;
                    let resp: HitokotoResult = raw_resp.json().await?;

                    let from_who = resp.from_who.unwrap_or_default();
                    OutgoingContent::Event(AnyMessageLikeEventContent::RoomMessage(
                        RoomMessageEventContent::text_html(
                            format!(
                                "『{0}』——{1}「{2}」\nFrom https://hitokoto.cn/?uuid={3}",
                                resp.hitokoto, from_who, resp.from, resp.uuid
                            ),
                            format!(
                            "<p><b>『{0}』</b><br/>——{1}「{2}」</p><p>From https://hitokoto.cn/?uuid={3}</p>",
                            resp.hitokoto, from_who, resp.from, resp.uuid
                            ),
                        ).make_reply_to(
                            &ev,
                            ForwardThread::No,
                            AddMentions::Yes,
                        ),
                    ))},
                None => OutgoingContent::None,
            };

            Ok(OutgoingResponse {
                room,
                content,
            })
    })
}
