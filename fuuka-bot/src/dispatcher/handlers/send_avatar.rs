use std::sync::Arc;

use file_format::FileFormat;
use matrix_sdk::media::{MediaFormat, MediaRequest};
use matrix_sdk::room::RoomMember;
use matrix_sdk::{ruma::events::room::message::OriginalRoomMessageEvent, Room};
use ruma::events::room::message::{
    AddMentions, ForwardThread, ImageMessageEventContent, MessageType, RoomMessageEventContent,
};
use ruma::events::room::{ImageInfo, MediaSource, ThumbnailInfo};
use ruma::events::AnyMessageLikeEventContent;
use ruma::{MxcUri, UInt};

use crate::RoomMemberExt;

use super::{Event, OutgoingContent, OutgoingResponse};

pub fn event_handler() -> super::EventHandler {
    dptree::case![Event::SendAvatar(member)].endpoint(
        |ev: Arc<OriginalRoomMessageEvent>,
         member: Arc<RoomMember>,
         client: matrix_sdk::Client,
         room: Arc<Room>| async move {
            let content = match member.avatar_url() {
                Some(avatar_url) => {
                    let name = member.name_or_id();
                    let info = get_image_info(avatar_url, &client).await?;
                    OutgoingContent::Event(AnyMessageLikeEventContent::RoomMessage(
                        RoomMessageEventContent::new(MessageType::Image(
                            ImageMessageEventContent::plain(
                                format!("[Avatar of {name}]"),
                                avatar_url.into(),
                            )
                            .info(Some(Box::new(info))),
                        ))
                        .make_reply_to(
                            &ev,
                            ForwardThread::No,
                            AddMentions::Yes,
                        ),
                    ))
                }
                None => OutgoingContent::Event(AnyMessageLikeEventContent::RoomMessage(
                    RoomMessageEventContent::text_plain("The user has no avatar."),
                )),
            };

            Ok(OutgoingResponse { room, content })
        },
    )
}

async fn get_image_info(
    avatar_url: &MxcUri,
    client: &matrix_sdk::Client,
) -> anyhow::Result<ImageInfo> {
    let request = MediaRequest {
        source: MediaSource::Plain(avatar_url.into()),
        format: MediaFormat::File,
    };
    let data = client.media().get_media_content(&request, false).await?;
    let dimensions = imagesize::blob_size(&data)?;
    let (width, height) = (dimensions.width, dimensions.height);
    let format = FileFormat::from_bytes(&data);
    let mimetype = format.media_type();
    let size = data.len();
    let mut thumb = ThumbnailInfo::new();
    let width = UInt::try_from(width)?;
    let height = UInt::try_from(height)?;
    let size = UInt::try_from(size)?;
    thumb.width = Some(width);
    thumb.height = Some(height);
    thumb.mimetype = Some(mimetype.to_string());
    thumb.size = Some(size);
    let mut info = ImageInfo::new();
    info.width = Some(width);
    info.height = Some(height);
    info.mimetype = Some(mimetype.to_string());
    info.size = Some(size);
    info.thumbnail_info = Some(Box::new(thumb));
    info.thumbnail_source = Some(MediaSource::Plain(avatar_url.into()));

    Ok(info)
}
