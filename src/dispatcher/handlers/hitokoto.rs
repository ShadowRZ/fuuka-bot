use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;
use matrix_sdk::ruma::events::room::message::{AddMentions, ForwardThread};

use crate::types::HitokotoResult;

use super::RequestType;

pub fn event_handler() -> super::EventHandler {
    dptree::case![RequestType::Hitokoto].endpoint(|
        request: super::IncomingRequest, injected: super::Injected| async move {
            if let Some(hitokoto) = injected.config.hitokoto() {
                let raw_resp = injected.http
                        .get(hitokoto)
                        .send()
                        .await?
                        .error_for_status()?;
                    let resp: HitokotoResult = raw_resp.json().await?;

                    let from_who = resp.from_who.unwrap_or_default();

                    request.room.send(RoomMessageEventContent::text_html(
                        format!(
                            "『{0}』——{1}「{2}」\nFrom https://hitokoto.cn/?uuid={3}",
                            resp.hitokoto, from_who, resp.from, resp.uuid
                        ),
                        format!(
                        "<p><b>『{0}』</b><br/>——{1}「{2}」</p><p>From https://hitokoto.cn/?uuid={3}</p>",
                        resp.hitokoto, from_who, resp.from, resp.uuid
                        ),
                    ).make_reply_to(
                        &request.ev,
                        ForwardThread::No,
                        AddMentions::Yes,
                    )).await?;
            }

            Ok(())
    })
}
