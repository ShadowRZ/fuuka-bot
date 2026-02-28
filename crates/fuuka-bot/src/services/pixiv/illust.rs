use std::str::FromStr;

use matrix_sdk::ruma::events::{
    Mentions,
    room::message::{OriginalRoomMessageEvent, TextMessageEventContent},
};
use matrix_sdk::{
    Room,
    attachment::{AttachmentConfig, AttachmentInfo, BaseImageInfo},
    room::reply::{EnforceThread, Reply},
    ruma::RoomId,
};
use mime::Mime;
use pixiv_ajax_api::{common::Restriction, illust::IllustInfo};

pub fn format(
    resp: IllustInfo,
    context: &super::Context,
    send_r18: bool,
    room_id: &RoomId,
    _prefix: bool,
) -> Option<(String, String)> {
    // R18 = 1, R18G = 2, General = 0
    let r18 = match resp.restriction {
        Restriction::General => false,
        Restriction::R18 => true,
        Restriction::R18G => true,
        _ => false,
    };
    if r18 && !send_r18 {
        return None;
    };

    use crate::format::ENVIRONMENT;
    use crate::format::pixiv::illust::{Author, Context, Tag};

    let tags: Vec<_> = resp
        .tags
        .tags
        .iter()
        .map(|tag| Tag {
            original: &tag.tag,
            translated: tag.translation.get("en").map(String::as_str),
        })
        .collect();

    let triggers = context
        .tag_triggers
        .check_for_tag_triggers(&resp.tags, room_id);

    let context = Context {
        id: resp.id,
        title: &resp.title,
        tags: &tags,
        author: Author {
            id: resp.user_id,
            name: &resp.user_name,
        },
        triggers: triggers.as_slice(),
    };

    let body: String = crate::format::pixiv::illust::text::format(&ENVIRONMENT, &context).ok()?;
    let html_body = crate::format::pixiv::illust::html::format(&ENVIRONMENT, &context).ok()?;

    Some((body, html_body))
}

pub async fn send(
    ev: &OriginalRoomMessageEvent,
    room: &Room,
    pixiv: &pixiv_ajax_api::PixivClient,
    http: &reqwest::Client,
    context: &super::Context,
    illust_id: i32,
    send_r18: bool,
) -> anyhow::Result<()> {
    let resp = pixiv.illust_info(illust_id).with_lang("zh").await?;
    let room_id = room.room_id();

    let url = resp.urls.original.clone();

    if let Some((body, formatted_body)) =
        crate::services::pixiv::illust::format(resp.clone(), context, send_r18, room_id, false)
    {
        use url::Url;

        let url = Url::parse(&url)?;
        let filename = url
            .path_segments()
            .and_then(|mut path| path.next_back())
            .unwrap_or("file.png")
            .to_string();

        let image = http
            .get(url)
            .header(reqwest::header::REFERER, "https://www.pixiv.net")
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await?
            .to_vec();

        let info = crate::matrix::imageinfo(&image)?;

        let config = AttachmentConfig::new()
            .info(AttachmentInfo::Image(BaseImageInfo {
                height: info.height,
                width: info.width,
                size: info.size,
                blurhash: None,
                is_animated: Some(false),
            }))
            .caption(Some(TextMessageEventContent::html(
                body.clone(),
                formatted_body.clone(),
            )))
            .mentions(Some(Mentions::with_user_ids([ev.sender.clone()])))
            .reply(Some(Reply {
                event_id: ev.event_id.clone(),
                enforce_thread: EnforceThread::MaybeThreaded,
            }));

        let content_type =
            Mime::from_str(file_format::FileFormat::from_bytes(&image).media_type())?;

        room.send_attachment(filename, &content_type, image.to_vec(), config)
            .await?;
    } else {
        tracing::debug!("Not sending response because the requested illust is marked R-18.");
    }

    Ok(())
}
