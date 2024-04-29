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

use anyhow::Context;
use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;
use url::{Host, Url};

use crate::{types::CrateMetadata, Error};

/// Dispatch prefixed messages that starts with `@Nahida`.
pub async fn dispatch(
    url: &Url,
    client: &reqwest::Client,
) -> anyhow::Result<Option<RoomMessageEventContent>> {
    match url.host() {
        Some(Host::Domain("crates.io")) => _crates_io(url, client).await,
        _ => Ok(None),
    }
}

async fn _crates_io(
    url: &Url,
    client: &reqwest::Client,
) -> anyhow::Result<Option<RoomMessageEventContent>> {
    let paths: Option<_> = url.path_segments().map(|s| s.collect::<Vec<_>>());
    let Some(paths) = paths else {
        return Result::Err(
            Error::UnexpectedError("No infomation can be extracted from URL.").into(),
        );
    };

    if paths.first() != Some(&"crates") {
        return Result::Err(
            Error::UnexpectedError("No infomation can be extracted from URL.").into(),
        );
    }

    let crate_name = paths.get(1).ok_or(Error::UnexpectedError(
        "No infomation can be extracted from URL.",
    ))?;
    let resp: CrateMetadata = client
        .get(format!("https://crates.io/api/v1/crates/{crate_name}"))
        .send()
        .await?
        .error_for_status()
        .context("Server reported failure")?
        .json()
        .await?;
    let version = paths
        .get(2)
        .map(|s| s.to_owned())
        .unwrap_or_else(|| &resp.crate_info.max_stable_version);

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
        .unwrap_or_else(|| format!("\nDocs: https://docs.rs/{crate_name}/{version}"));
    let version_info = resp.versions.iter().find(|i| i.num == version);

    let msrv_str = version_info
        .and_then(|info| info.rust_version.as_ref())
        .map(|msrv| format!("\nMSRV: {msrv}",))
        .unwrap_or_default();

    Ok(Some(RoomMessageEventContent::text_html(
        format!("[Rust Crate] {name} v{version}: {desc}\n{msrv_str}{docs}{repo}"),
        format!("<p><b>[Rust Crate]</b> {name} v{version}: {desc}</p><p>{msrv_str}<br/>{docs}<br/>{repo}</p>")
    )))
}
