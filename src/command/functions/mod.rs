pub(super) mod member_stream;
pub(super) mod quote;

use std::{
    collections::{HashMap, HashSet},
    io::{Cursor, Read},
    path::Path,
    sync::Arc,
};

use file_format::{FileFormat, Kind};
use matrix_sdk::{
    media::{MediaFormat, MediaRequest},
    ruma::{
        events::room::{ImageInfo, MediaSource, ThumbnailInfo},
        MxcUri, UInt,
    },
    Client, Media,
};
use mime::Mime;
use tokio::task::JoinSet;
use zip::ZipArchive;

use crate::{
    events::sticker::{RoomStickerEventContent, StickerData, StickerPack, StickerUsage},
    Error,
};

#[tracing::instrument(skip(client, data), err)]
pub(super) async fn prepare_sticker_upload_event_content(
    client: &Client,
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

#[tracing::instrument(skip(client), err)]
pub(super) async fn get_image_info(
    avatar_url: &MxcUri,
    client: &Client,
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
