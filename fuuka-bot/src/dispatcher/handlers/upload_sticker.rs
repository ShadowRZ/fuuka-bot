use std::collections::{HashMap, HashSet};
use std::io::Cursor;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;

use file_format::{FileFormat, Kind};
use matrix_sdk::ruma::events::room::message::MessageType;
use matrix_sdk::ruma::events::room::message::{
    AddMentions, ForwardThread, RoomMessageEventContent,
};
use matrix_sdk::ruma::events::room::ImageInfo;
use matrix_sdk::ruma::events::AnyMessageLikeEventContent;
use matrix_sdk::ruma::events::{AnyMessageLikeEvent, AnyTimelineEvent, MessageLikeEvent};
use matrix_sdk::ruma::UInt;
use matrix_sdk::Media;
use matrix_sdk::{ruma::events::room::message::OriginalRoomMessageEvent, Room};
use mime::Mime;
use tokio::task::JoinSet;
use zip::ZipArchive;

use crate::events::sticker::{RoomStickerEventContent, StickerData, StickerPack, StickerUsage};
use crate::Error;

use super::{Event, OutgoingContent, OutgoingResponse};

pub fn event_handler() -> super::EventHandler {
    dptree::case![Event::UploadSticker {
        ev,
        pack_name,
        sticker_room
    }]
    .endpoint(
        |(ref_ev, pack_name, sticker_room): (AnyTimelineEvent, String, Arc<Room>),
         trigger_ev: Arc<OriginalRoomMessageEvent>,
         room: Arc<Room>| async move {
            let content = match ref_ev {
                AnyTimelineEvent::MessageLike(AnyMessageLikeEvent::RoomMessage(
                    MessageLikeEvent::Original(ev),
                )) => {
                    let content = ev.content;
                    match content.msgtype {
                        MessageType::File(event_content) => {
                            let name = event_content
                                .filename
                                .clone()
                                .unwrap_or(format!("{}", ev.origin_server_ts.0));
                            let data = room
                                .client()
                                .media()
                                .get_file(&event_content, false)
                                .await?
                                .ok_or(Error::UnexpectedError("File has no data!"))?;
                            let format = FileFormat::from_bytes(&data);
                            let mimetype = format.media_type();
                            if mimetype != "application/zip" {
                                return Result::Err(
                                    Error::UnexpectedError("File is not a ZIP file!").into(),
                                );
                            }
                            let content = prepare_sticker_upload_event_content(
                                &room.client(),
                                data,
                                pack_name,
                            )
                            .await?;
                            sticker_room
                                .send_state_event_for_key(&name, content)
                                .await?;
                            Some(RoomMessageEventContent::text_plain("Done.").make_reply_to(
                                &trigger_ev,
                                ForwardThread::No,
                                AddMentions::Yes,
                            ))
                        }
                        _ => None,
                    }
                }
                _ => None,
            };

            Ok(OutgoingResponse {
                room,
                content: content
                    .map(|c| OutgoingContent::Event(AnyMessageLikeEventContent::RoomMessage(c)))
                    .unwrap_or_default(),
            })
        },
    )
}

async fn prepare_sticker_upload_event_content(
    client: &matrix_sdk::Client,
    data: Vec<u8>,
    display_name: String,
) -> anyhow::Result<RoomStickerEventContent> {
    let media: Arc<Media> = Arc::new(client.media());
    let mut set = JoinSet::new();
    let data = Cursor::new(data);
    let mut archive = ZipArchive::new(data)?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        if !entry.is_file() {
            continue;
        }
        let path = Path::new(entry.name()).to_owned();
        let Some(name) = path
            .file_name()
            .and_then(|data| data.to_str())
            .map(ToString::to_string)
        else {
            continue;
        };
        let mut data = Vec::new();
        entry.read_to_end(&mut data)?;
        let format = FileFormat::from_bytes(&data);
        if format.kind() != Kind::Image {
            continue;
        }
        let mimetype = format.media_type();
        let mime = mimetype.parse::<Mime>()?;
        let dimensions = imagesize::blob_size(&data)?;
        let (width, height) = (dimensions.width, dimensions.height);
        let mut info = ImageInfo::new();
        let width = UInt::try_from(width)?;
        let height = UInt::try_from(height)?;
        let size = data.len();
        let size = UInt::try_from(size)?;
        info.width = Some(width);
        info.height = Some(height);
        info.mimetype = Some(mimetype.to_string());
        info.size = Some(size);

        let media = media.clone();
        set.spawn(async move {
            match media.upload(&mime, data).await {
                Ok(resp) => Some((name, resp.content_uri, info)),
                Err(e) => {
                    tracing::error!("Unexpected error while uploading '{name}': {e:#}");
                    None
                }
            }
        });
    }

    let mut images = HashMap::new();
    while let Some(res) = set.join_next().await {
        if let Some((name, url, info)) = res? {
            images.insert(name, StickerData { url, info });
        }
    }
    let avatar_url = images
        .values()
        .next()
        .map(|data| data.url.clone())
        .ok_or(Error::UnexpectedError("No image was uploaded!"))?;
    Ok(RoomStickerEventContent {
        images,
        pack: StickerPack {
            avatar_url,
            display_name,
            usage: HashSet::from([StickerUsage::Sticker]),
        },
    })
}
