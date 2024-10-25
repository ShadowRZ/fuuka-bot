use std::{str::FromStr, sync::OnceLock};

use anyhow::{Context, Ok};
use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;
use mime::Mime;
use regex::Regex;
use url::Url;

use crate::types::{BiliBiliVideo, BiliBiliVideoStat};

static REGEX_CELL: OnceLock<Regex> = OnceLock::new();

pub async fn bilibili_video(
    client: &reqwest::Client,
    url: Url,
) -> anyhow::Result<Option<RoomMessageEventContent>> {
    let resp = client
        .get(url)
        .send()
        .await?
        .error_for_status()
        .context("Server reported failure")?;
    let headers = resp.headers();
    let content_type = headers.get(reqwest::header::CONTENT_TYPE);

    match content_type {
        Some(content_type) => {
            let content_type = Mime::from_str(content_type.to_str()?)?;
            if (content_type.type_(), content_type.subtype()) == (mime::TEXT, mime::HTML) {
                parse_initial_state(&resp.text().await?).map(|ok| ok.map(create_message_content))
            } else {
                Ok(None)
            }
        }
        None => Ok(None),
    }
}

fn parse_initial_state(input: &str) -> anyhow::Result<Option<BiliBiliVideo>> {
    let regex = REGEX_CELL.get_or_init(|| {
        Regex::new(r"(?:window\.__INITIAL_STATE__\s*=)\s*(?P<json>\{(?s:.+)\})\s*(?:;)").unwrap()
    });
    let captures = regex
        .captures(input)
        .ok_or(crate::Error::UnexpectedError("No captures!"))?;
    let json_str = &captures["json"];
    Ok(serde_json::from_str(json_str)?)
}

fn create_message_content(data: BiliBiliVideo) -> RoomMessageEventContent {
    let title = &data.video_data.title;
    let bvid = &data.video_data.bvid;
    let owner_name = &data.video_data.owner.name;
    let owner_uid = &data.video_data.owner.mid;

    let BiliBiliVideoStat {
        view,
        danmaku,
        favorite,
        coin,
        share,
        like,
        ..
    } = &data.video_data.stat;
    let stats_str = format!("Views: {view} Likes: {like} Coins: {coin} Favorties: {favorite} Shares: {share} Danmakus: {danmaku}");
    RoomMessageEventContent::text_html(
        format!("[BiliBili/Video] {title} https://www.bilibili.com/video/{bvid}\nUP: {owner_name} https://space.bilibili.com/{owner_uid}\n{stats_str}"),
        format!(
            "<p><b>[BiliBili/Video]</b> <a href='https://www.bilibili.com/video/{bvid}'>{title}</a></p><p>UP: <a href='https://space.bilibili.com/{owner_uid}'>{owner_name}</a></p><p>{stats_str}</p>"
        ),
    )
}
