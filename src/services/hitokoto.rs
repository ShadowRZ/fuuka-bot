use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;
use serde::{Deserialize, Serialize};
use url::Url;

/// <https://developer.hitokoto.cn/sentence/#%E8%BF%94%E5%9B%9E%E4%BF%A1%E6%81%AF>
#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(missing_docs)]
pub struct Response {
    pub id: u64,
    pub uuid: String,
    pub hitokoto: String,
    #[serde(rename = "type")]
    pub type_: Type,
    pub from: String,
    pub from_who: Option<String>,
    pub creator: String,
    pub creator_uid: u64,
    pub reviewer: u64,
    pub commit_from: String,
    pub created_at: String,
    length: u64,
}

/// <https://developer.hitokoto.cn/sentence/#%E5%8F%A5%E5%AD%90%E7%B1%BB%E5%9E%8B-%E5%8F%82%E6%95%B0>
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Type {
    /// 动画
    #[serde(rename = "a")]
    Anime,
    /// 漫画
    #[serde(rename = "b")]
    Comic,
    /// 游戏
    #[serde(rename = "c")]
    Game,
    /// 文学
    #[serde(rename = "d")]
    Literature,
    /// 原创
    #[serde(rename = "e")]
    Original,
    /// 来自网络
    #[serde(rename = "f")]
    Internet,
    /// 其他
    #[serde(rename = "g")]
    Other,
    /// 影视
    #[serde(rename = "h")]
    Video,
    /// 诗词
    #[serde(rename = "i")]
    Poetry,
    /// 网易云
    #[serde(rename = "j")]
    NetEase,
    /// 哲学
    #[serde(rename = "k")]
    Philosophy,
    /// 抖机灵
    #[serde(rename = "l")]
    Joke,
}

pub fn format(resp: Response) -> RoomMessageEventContent {
    let from_who = resp.from_who.unwrap_or_default();

    RoomMessageEventContent::text_html(
        format!(
            "『{0}』——{1}「{2}」\nFrom https://hitokoto.cn/?uuid={3}",
            resp.hitokoto, from_who, resp.from, resp.uuid
        ),
        format!(
            "<p><b>『{0}』</b><br/>——{1}「{2}」</p><p>From https://hitokoto.cn/?uuid={3}</p>",
            resp.hitokoto, from_who, resp.from, resp.uuid
        ),
    )
}

pub async fn request(client: &reqwest::Client, base: Url) -> anyhow::Result<Response> {
    tracing::Span::current().record(
        "fuuka_bot.hitokoto.base_url",
        tracing::field::display(&base),
    );
    let raw = client.get(base).send().await?.error_for_status()?;
    let resp: Response = raw.json().await?;

    Ok(resp)
}
