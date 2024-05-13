use matrix_sdk::ruma::html::remove_html_reply_fallback;
use matrix_sdk::{
    room::RoomMember,
    ruma::events::{
        room::message::{
            sanitize::remove_plain_reply_fallback, MessageType, Relation, RoomMessageEventContent,
        },
        sticker::StickerEventContent,
        AnyMessageLikeEvent, AnyMessageLikeEventContent, AnyTimelineEvent, MessageLikeEvent,
    },
};

use crate::{Context, MxcUriExt, RoomMemberExt};

impl Context {
    #[tracing::instrument(
        skip(self),
        fields(
            event_id = %self.ev.event_id,
            room_id = %self.room.room_id()
        ),
        err
    )]
    pub(super) async fn _quote(
        &self,
        ev: AnyTimelineEvent,
        member: RoomMember,
    ) -> anyhow::Result<Option<AnyMessageLikeEventContent>> {
        let room_id = &self.ev.room_id;
        match ev {
            AnyTimelineEvent::MessageLike(AnyMessageLikeEvent::RoomMessage(
                MessageLikeEvent::Original(ev),
            )) => {
                let ev = ev
                    .unsigned
                    .relations
                    .replace
                    .clone()
                    .map(|ev| ev.into_full_event(room_id.clone()))
                    .unwrap_or(ev);
                let content = ev.content;
                let replace_content = content
                    .relates_to
                    .clone()
                    .and_then(|rel| match rel {
                        Relation::Replacement(content) => Some(content),
                        _ => None,
                    })
                    .map(|replacement| replacement.new_content);
                let content = replace_content.unwrap_or(content.into());
                match content.msgtype {
                    MessageType::Text(content) => {
                        let string = format!(
                            "<span size=\"larger\" foreground=\"#1f4788\">{}</span>\n{}",
                            member.name_or_id(),
                            content
                                .formatted
                                .map(|formatted| super::functions::quote::html2pango(
                                    &remove_html_reply_fallback(&formatted.body)
                                ))
                                .transpose()?
                                .unwrap_or(
                                    html_escape::encode_text(remove_plain_reply_fallback(
                                        &content.body
                                    ))
                                    .to_string()
                                )
                        );
                        let data = super::functions::quote::quote(
                            member
                                .avatar_url()
                                .map(|url| url.http_url(&self.homeserver))
                                .transpose()?
                                .map(|s| s.to_string()),
                            &string,
                        )
                        .await?;
                        let mime: mime::Mime = "image/webp".parse()?;
                        let resp = self.room.client().media().upload(&mime, data).await?;
                        let client = &self.room.client();
                        let info =
                            super::functions::get_image_info(&resp.content_uri, client).await?;
                        let send_content =
                            StickerEventContent::new("[Quote]".to_string(), info, resp.content_uri);
                        Ok(Some(AnyMessageLikeEventContent::Sticker(send_content)))
                    }
                    _ => Ok(Some(AnyMessageLikeEventContent::RoomMessage(
                        RoomMessageEventContent::text_plain(format!(
                            "Unsupported event type, event type in Rust: {}",
                            std::any::type_name_of_val(&content.msgtype)
                        )),
                    ))),
                }
            }
            _ => Ok(Some(AnyMessageLikeEventContent::RoomMessage(
                RoomMessageEventContent::text_plain(format!(
                    "Unsupported event type, event type in Rust: {}",
                    std::any::type_name_of_val(&ev)
                )),
            ))),
        }
    }
}
