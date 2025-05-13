use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;
use serde::{Deserialize, Serialize};
use url::Url;

/// <https://developer.hitokoto.cn/sentence/#%E8%BF%94%E5%9B%9E%E4%BF%A1%E6%81%AF>
#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(missing_docs)]
pub struct Response {
    pub uuid: String,
    pub hitokoto: String,
    pub from: String,
    pub from_who: Option<String>,
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

#[tracing::instrument(name = "hitokoto", skip_all, err)]
pub async fn request(client: &reqwest::Client, base: Url) -> anyhow::Result<Response> {
    let raw = client.get(base).send().await?.error_for_status()?;
    let resp: Response = raw.json().await?;

    Ok(resp)
}
