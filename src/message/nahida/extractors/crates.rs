//! Extracts crates.io URLs.

use anyhow::Context;
use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;

use crate::types::CrateMetadata;

#[tracing::instrument(name = "crates", skip(client), err)]
pub async fn crates_crate(
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
        format!(
            "<p><b>[Rust/Crate]</b> {name} v{version}: {desc}</p><p>{msrv_str}<br/>{docs}<br/>{repo}</p>"
        ),
    )))
}
