use std::{io::Cursor, str::FromStr};

use crate::{RoomExt, message::Injected};
use image::ImageFormat;
use matrix_sdk::ruma::{
    UInt,
    events::{
        relation::InReplyTo,
        room::{MediaSource, ThumbnailInfo, message::Relation},
    },
};
use matrix_sdk::{
    Room,
    event_handler::Ctx,
    ruma::events::{
        AnyMessageLikeEvent, AnyTimelineEvent,
        room::{
            ImageInfo,
            message::{MessageType, OriginalRoomMessageEvent, RoomMessageEvent},
        },
        sticker::StickerEventContent,
    },
};
use mime::Mime;

pub async fn process(
    ev: &OriginalRoomMessageEvent,
    room: &Room,
    injected: &Ctx<Injected>,
) -> anyhow::Result<()> {
    let _ = injected;

    let event_id = ev.event_id.clone();
    let event = room.in_reply_to_event(ev).await?;

    if let Some(AnyTimelineEvent::MessageLike(AnyMessageLikeEvent::RoomMessage(
        RoomMessageEvent::Original(event),
    ))) = event
    {
        if let MessageType::Text(content) = event.content.msgtype {
            let member =
                room.get_member(&event.sender)
                    .await?
                    .ok_or(crate::Error::UnexpectedError(
                        "matrix-rust-sdk couldn't found the member in the room??",
                    ))?;
            let image = crate::render::quote::render(content, &member).await?;
            let mut buf = Cursor::new(Vec::new());
            image.write_to(&mut buf, ImageFormat::WebP)?;

            let data = buf.into_inner();
            let size = data.len();
            let res = room
                .client()
                .media()
                .upload(&Mime::from_str("image/webp")?, data, None)
                .await?;

            let (width, height) = (image.width(), image.height());

            let mxc = res.content_uri;
            let mimetype = "image/webp".to_string();
            let mut thumb = ThumbnailInfo::new();
            thumb.width = Some(width.into());
            thumb.height = Some(height.into());
            thumb.mimetype = Some(mimetype.clone());
            thumb.size = Some(UInt::try_from(size)?);
            let mut info = ImageInfo::new();
            info.width = Some(width.into());
            info.height = Some(height.into());
            info.mimetype = Some(mimetype);
            info.size = Some(UInt::try_from(size)?);
            info.thumbnail_info = Some(Box::new(thumb));
            info.thumbnail_source = Some(MediaSource::Plain(mxc.clone()));
            let mut content = StickerEventContent::new("[Quote]".to_string(), info, mxc);
            content.relates_to = Some(Relation::Reply {
                in_reply_to: InReplyTo::new(event_id),
            });
            room.send(content).await?;
        }
    }

    Ok(())
}
