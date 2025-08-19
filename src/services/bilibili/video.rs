use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;
use regex::Regex;
use reqwest::header::HeaderValue;
use serde::Deserialize;
use std::sync::LazyLock;
use time::OffsetDateTime;

static REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:window\.__INITIAL_STATE__\s*=)\s*(?P<json>\{(?s:.+)\})\s*(?:;)").unwrap()
});

static USER_AGENT_HEADER_VALUE: LazyLock<HeaderValue> =
    LazyLock::new(|| HeaderValue::from_static(super::USER_AGENT));

pub async fn request(client: &reqwest::Client, id: &str) -> anyhow::Result<Video> {
    let url = format!("https://www.bilibili.com/video/{id}");
    let body = client
        .get(url)
        .header(reqwest::header::USER_AGENT, USER_AGENT_HEADER_VALUE.clone())
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    let captures = REGEX
        .captures(&body)
        .ok_or(crate::Error::UnexpectedError("No captures!"))?; // TODO: Better errors
    let json_str = &captures["json"];

    Ok(serde_json::from_str(json_str)?)
}

pub fn format(resp: Video, prefix: bool) -> RoomMessageEventContent {
    let title = &resp.data.title;
    let bvid = &resp.data.bvid;
    let owner_name = &resp.data.owner.name;
    let owner_uid = &resp.data.owner.mid;

    let Stat {
        view,
        danmaku,
        favorite,
        coin,
        share,
        like,
        ..
    } = &resp.data.stat;
    let stats_str: String = format!(
        "Views: {view} Likes: {like} Coins: {coin} Favorties: {favorite} Shares: {share} Danmakus: {danmaku}"
    );
    let prefix_text = if prefix { "[BiliBili/Video] " } else { "" };
    let prefix_html = if prefix {
        "<p><b>[BiliBili/Video]</b> "
    } else {
        ""
    };
    RoomMessageEventContent::text_html(
        format!(
            "{prefix_text}{title} https://www.bilibili.com/video/{bvid}\nUP: {owner_name} https://space.bilibili.com/{owner_uid}\n{stats_str}"
        ),
        format!(
            "{prefix_html}<a href='https://www.bilibili.com/video/{bvid}'>{title}</a></p><p>UP: <a href='https://space.bilibili.com/{owner_uid}'>{owner_name}</a></p><p>{stats_str}</p>"
        ),
    )
}

#[derive(Deserialize, Debug, Clone)]
#[allow(missing_docs)]
#[serde(rename_all = "camelCase")]
pub struct Video {
    #[serde(rename = "videoData", alias = "videoInfo")]
    pub data: VideoData,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(missing_docs)]
#[serde(rename_all = "camelCase")]
pub struct VideoData {
    pub bvid: String,
    pub title: String,
    pub desc: String,
    #[serde(with = "time::serde::timestamp::option")]
    pub pubdate: Option<OffsetDateTime>,
    pub owner: Owner,
    pub stat: Stat,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(missing_docs)]
#[serde(rename_all = "camelCase")]
pub struct Stat {
    pub view: i128,
    pub danmaku: i128,
    pub reply: i128,
    pub favorite: i128,
    pub coin: i128,
    pub share: i128,
    pub like: i128,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(missing_docs)]
#[serde(rename_all = "camelCase")]
pub struct Owner {
    pub mid: i128,
    pub name: String,
}
