use crate::Context;
use matrix_sdk::ruma::events::{
    room::message::RoomMessageEventContent, AnyMessageLikeEventContent,
};

use super::functions::nixpkgs_pr::fetch_nixpkgs_pr;

impl Context {
    pub(super) async fn _nixpkgs(
        &self,
        pr_number: u64,
    ) -> anyhow::Result<Option<AnyMessageLikeEventContent>> {
        let Some(ref nixpkgs_pr) = self.config.nixpkgs_pr else {
            return Ok(None);
        };
        let client = &self.http;
        let result = fetch_nixpkgs_pr(client, &nixpkgs_pr.token, pr_number).await?;

        let in_branches = result.in_branches.as_ref().map(|in_branches| {
            format!(
                "\nstaging-next {staging} master {master} nixos-unstable-small {nixos_unstable_small} nixpkgs-unstable {nixpkgs_unstable} nixos-unstable {nixos_unstable}",
                staging = if in_branches.staging { "✅" } else { "❎" },
                master = if in_branches.master { "✅" } else { "❎" },
                nixos_unstable_small = if in_branches.nixos_unstable_small { "✅" } else { "❎" },
                nixpkgs_unstable = if in_branches.nixpkgs_unstable { "✅" } else { "❎" },
                nixos_unstable = if in_branches.nixos_unstable { "✅" } else { "❎" },
            )
        }).unwrap_or_default();
        let in_branches_html = result.in_branches.as_ref().map(|in_branches| {
            format!(
                "<p><b>staging-next</b> {staging}<br/><b>master</b> {master}<br/><b>nixos-unstable-small</b> {nixos_unstable_small}<br/><b>nixpkgs-unstable</b> {nixpkgs_unstable}<br/><b>nixos-unstable</b> {nixos_unstable}</p>",
                staging = if in_branches.staging { "✅" } else { "❎" },
                master = if in_branches.master { "✅" } else { "❎" },
                nixos_unstable_small = if in_branches.nixos_unstable_small { "✅" } else { "❎" },
                nixpkgs_unstable = if in_branches.nixpkgs_unstable { "✅" } else { "❎" },
                nixos_unstable = if in_branches.nixos_unstable { "✅" } else { "❎" },
            )
        }).unwrap_or_default();

        Ok(Some(AnyMessageLikeEventContent::RoomMessage(
            RoomMessageEventContent::text_html(
                format!(
                    "PR #{pr_number}: {title} https://github.com/NixOS/nixpkgs/pull/{pr_number}{in_branches}",
                    title = result.title,
                    in_branches = in_branches,
                ),
                format!(
                    "<p><a href='https://github.com/NixOS/nixpkgs/pull/{pr_number}'>PR #{pr_number}: {title}</a>{in_branches}",
                    title = result.title,
                    in_branches = in_branches_html,
                ),
            ),
        )))
    }
}
