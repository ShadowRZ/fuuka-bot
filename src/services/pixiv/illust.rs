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
use pixrs::{IllustInfo, Restriction};

use crate::config::PixivConfig;

pub fn format(
    resp: IllustInfo,
    config: &PixivConfig,
    send_r18: bool,
    room_id: &RoomId,
    prefix: bool,
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
    let tag_str = resp
        .tags
        .tags
        .iter()
        .map(|tag| {
            format!(
                "#{tag}{translated}",
                tag = tag.tag,
                translated = tag
                    .translation
                    .get("en")
                    .map(|s| format!(" ({s})"))
                    .unwrap_or_default()
            )
        })
        .collect::<Vec<String>>()
        .join(" ");
    let tag_html_str = resp
        .tags
        .tags
        .iter()
        .map(|tag| {
            format!(
                "<font color='#3771bb'>#{tag}</font>{translated}",
                tag = tag.tag,
                translated = tag
                    .translation
                    .get("en")
                    .map(|s| format!(" ({s})"))
                    .unwrap_or_default()
            )
        })
        .collect::<Vec<String>>()
        .join(" ");
    // Specials
    let specials_str = config
        .traps
        .check_for_traps(&resp.tags, room_id)
        .map(|s| format!("\n#{s}诱捕器"))
        .unwrap_or_default();
    let specials_str_html = config
        .traps
        .check_for_traps(&resp.tags, room_id)
        .map(|s| format!("<p><b><font color='#d72b6d'>#{s}诱捕器</font></b></p>"))
        .unwrap_or_default();
    let body = format!(
        "{prefix}{title} https://www.pixiv.net/artworks/{id}\n{tag_str}\nAuthor: {author}{specials_str}",
        prefix = if prefix { "[Pixiv/Illust] " } else { "" },
        title = resp.title,
        id = resp.id,
        author = resp.user_name
    );
    let html_body = format!(
        "{prefix}<a href='https://www.pixiv.net/artworks/{id}'>{title}</a></p><p>{tag_html_str}</p><p>Author: {author}</p>{specials_str_html}",
        prefix = if prefix {
            "<p><b>[Pixiv/Illust]</b> "
        } else {
            ""
        },
        title = resp.title,
        id = resp.id,
        author = resp.user_name
    );

    Some((body, html_body))
}

pub async fn send(
    ev: &OriginalRoomMessageEvent,
    room: &Room,
    pixiv: &pixrs::PixivClient,
    http: &reqwest::Client,
    config: &PixivConfig,
    illust_id: i32,
    send_r18: bool,
) -> anyhow::Result<()> {
    let resp = pixiv.illust_info(illust_id).with_lang("zh").await?;
    let room_id = room.room_id();

    let url = resp.urls.original.clone();

    if let Some((body, formatted_body)) =
        crate::services::pixiv::illust::format(resp.clone(), config, send_r18, room_id, false)
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

        let info = crate::imageinfo(&image)?;

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
