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
        .ok_or(anyhow::anyhow!("No captures!"))?; // TODO: Better errors
    let json_str = &captures["json"];

    Ok(serde_json::from_str(json_str)?)
}

pub fn format(resp: Video, _prefix: bool) -> anyhow::Result<RoomMessageEventContent> {
    use crate::format::ENVIRONMENT;
    use crate::format::bilibili::video::{Author, Context, Counts};

    let tags: Vec<_> = resp.tags.iter().map(|tag| tag.tag_name.as_str()).collect();

    let Stat {
        view,
        danmaku,
        favorite,
        coin,
        share,
        like,
        reply,
        ..
    } = resp.data.stat;
    let context = Context {
        id: &resp.data.bvid,
        title: &resp.data.title,
        description: match resp.data.desc.as_str() {
            "-" => None,
            desc => Some(desc),
        },
        tags: &tags,
        author: Author {
            id: resp.data.owner.mid,
            name: &resp.data.owner.name,
        },
        counts: Counts {
            view,
            like,
            coin,
            favorite,
            danmaku,
            reply,
            share,
        },
    };
    let body: String = crate::format::bilibili::video::text::format(&ENVIRONMENT, &context)?;
    let html_body = crate::format::bilibili::video::html::format(&ENVIRONMENT, &context)?;

    Ok(RoomMessageEventContent::text_html(body, html_body))
}

#[derive(Deserialize, Debug, Clone)]
#[allow(missing_docs)]
#[serde(rename_all = "camelCase")]
pub struct Video {
    #[serde(rename = "videoData", alias = "videoInfo")]
    pub data: VideoData,
    pub tags: Vec<Tag>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Tag {
    pub tag_name: String,
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
    pub view: u64,
    pub danmaku: u64,
    pub reply: u64,
    pub favorite: u64,
    pub coin: u64,
    pub share: u64,
    pub like: u64,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(missing_docs)]
#[serde(rename_all = "camelCase")]
pub struct Owner {
    pub mid: u64,
    pub name: String,
}
