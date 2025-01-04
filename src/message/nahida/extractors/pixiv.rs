//! Extracts Pixiv URLs.

use matrix_sdk::ruma::{events::room::message::RoomMessageEventContent, RoomId};
use pixrs::Restriction;

use crate::config::PixivConfig;

pub async fn pixiv_illust(
    pixiv: &pixrs::PixivClient,
    artwork_id: i32,
    config: &PixivConfig,
    room_id: &RoomId,
) -> anyhow::Result<Option<RoomMessageEventContent>> {
    let resp = pixiv.illust_info(artwork_id).with_lang("zh").await?;
    // R18 = 1, R18G = 2, General = 0
    let r18 = match resp.restriction {
        Restriction::General => false,
        Restriction::R18 => true,
        Restriction::R18G => true,
        _ => false,
    };
    let r18_enabled = config.r18;
    if r18 && !r18_enabled {
        return Ok(None);
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
        "[Pixiv/Illust] {title} https://www.pixiv.net/artworks/{id}\n{tag_str}\nAuthor: {author}{specials_str}",
        title = resp.title,
        id = resp.id,
        author = resp.user_name
    );
    let html_body = format!(
        "<p><b>[Pixiv/Illust]</b> <a href='https://www.pixiv.net/artworks/{id}'>{title}</a></p><p>{tag_html_str}</p><p>Author: {author}</p>{specials_str_html}",
        title = resp.title,
        id = resp.id,
        author = resp.user_name
    );
    Ok(Some(RoomMessageEventContent::text_html(body, html_body)))
}
