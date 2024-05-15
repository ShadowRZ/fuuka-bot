//! Handler for prefixed messages that starts with `@Nahida`.
//!
//! > [Nahida](https://genshin-impact.fandom.com/wiki/Nahida) is a character from _Genshin Impact_.
//!
//! ## Usage
//!
//! Send `@Nahida` followed by a supported URL, example:
//!
//! ```text
//! # Outputs infomation for Rust crate syn
//! @Nahida https://crates.io/crates/syn
//! ```

mod link_type;

use anyhow::Context;
use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;
use matrix_sdk::ruma::RoomId;
use url::Url;

use crate::{config::PixivConfig, types::CrateMetadata};

use self::link_type::{CrateLinkType, LinkType, PixivLinkType};

/// Dispatch prefixed messages that starts with `@Nahida`.
pub async fn dispatch(
    url: Url,
    ctx: &crate::Context,
    // client: &reqwest::Client,
    // pixiv: Option<&pixrs::PixivClient>,
) -> anyhow::Result<Option<RoomMessageEventContent>> {
    let client = &ctx.http;
    let pixiv = ctx.pixiv.as_deref();
    match url.try_into()? {
        LinkType::Crates(CrateLinkType::CrateInfo { name, version }) => {
            _crates_io(name, version, client).await
        }
        LinkType::Pixiv(PixivLinkType::Artwork(artwork_id)) => match pixiv {
            Some(pixiv) => _pixiv(pixiv, artwork_id, &ctx.config.pixiv, ctx.room.room_id()).await,
            None => Ok(None),
        },
        LinkType::Generic => Ok(None), // TODO
        LinkType::CannotBeABase => {
            Result::Err(crate::Error::UnexpectedError("URL is a cannot-be-a-base!").into())
        }
    }
}

async fn _crates_io(
    name: String,
    version: Option<String>,
    client: &reqwest::Client,
) -> anyhow::Result<Option<RoomMessageEventContent>> {
    let resp: CrateMetadata = client
        .get(format!("https://crates.io/api/v1/crates/{name}"))
        .send()
        .await?
        .error_for_status()
        .context("Server reported failure")?
        .json()
        .await?;
    let version = version
        .as_ref()
        .unwrap_or(&resp.crate_info.max_stable_version);

    let name = resp.crate_info.name;
    let desc = resp
        .crate_info
        .description
        .unwrap_or("(No Description)".to_string());
    let repo = resp
        .crate_info
        .repository
        .map(|s| format!("\nRepository: {s}"))
        .unwrap_or_default();
    let docs = resp
        .crate_info
        .documentation
        .map(|s| format!("\nDocs: {s}"))
        .unwrap_or_else(|| format!("\nDocs: https://docs.rs/{name}/{version}"));
    let version_info = resp.versions.iter().find(|i| i.num == *version);

    let msrv_str = version_info
        .and_then(|info| info.rust_version.as_ref())
        .map(|msrv| format!("\nMSRV: {msrv}",))
        .unwrap_or_default();

    Ok(Some(RoomMessageEventContent::text_html(
        format!("[Rust/Crate] {name} v{version}: {desc}\n{msrv_str}{docs}{repo}"),
        format!("<p><b>[Rust/Crate]</b> {name} v{version}: {desc}</p><p>{msrv_str}<br/>{docs}<br/>{repo}</p>")
    )))
}

async fn _pixiv(
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
        "[Pixiv/Illust] {title} https://pixiv.net/i/{id}\n{tag_str}\nAuthor: {author}{specials_str}",
        title = resp.title,
        id = resp.id,
        author = resp.user_name
    );
    let html_body = format!(
        "<p><b>[Pixiv/Illust]</b> <a href='https://pixiv.net/i/{id}'>{title}</a></p><p>{tag_html_str}</p><p>Author: {author}</p>{specials_str_html}",
        title = resp.title,
        id = resp.id,
        author = resp.user_name
    );
    Ok(Some(RoomMessageEventContent::text_html(body, html_body)))
}
