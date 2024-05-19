//! Extracts Pixiv URLs.

use matrix_sdk::ruma::{events::room::message::RoomMessageEventContent, RoomId};

use crate::config::PixivConfig;

pub async fn pixiv_illust(
    pixiv: &pixrs::PixivClient,
    artwork_id: i32,
    config: &PixivConfig,
    room_id: &RoomId,
) -> anyhow::Result<Option<RoomMessageEventContent>> {
    let resp = pixiv.illust_info(artwork_id).await?;
    let tag_str = resp
        .tags
        .tags
        .iter()
        .map(|tag| format!("#{tag}", tag = tag.tag))
        .collect::<Vec<String>>()
        .join(" ");
    let tag_html_str = resp
        .tags
        .tags
        .iter()
        .map(|tag| format!("<font color='#3771bb'>#{tag}</font>", tag = tag.tag))
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
        "[Pixiv/Illust] {title} https://pixiv.net/artworks/{id}\n{tag_str}\nAuthor: {author}{specials_str}",
        title = resp.title,
        id = resp.id,
        author = resp.user_name
    );
    let html_body = format!(
        "<p><b>[Pixiv/Illust]</b> <a href='https://pixiv.net/artworks/{id}'>{title}</a></p><p>{tag_html_str}</p><p>Author: {author}</p>{specials_str_html}",
        title = resp.title,
        id = resp.id,
        author = resp.user_name
    );
    Ok(Some(RoomMessageEventContent::text_html(body, html_body)))
}
