use std::time::Duration;

use crate::Context;
use futures_util::{pin_mut, StreamExt};
use matrix_sdk::ruma::events::{
    room::message::RoomMessageEventContent, AnyMessageLikeEventContent,
};

use super::functions::nixpkgs_pr::fetch_nixpkgs_pr;

impl Context {
    pub(super) async fn _nixpkgs(
        &self,
        pr_number: i32,
        track: bool,
    ) -> anyhow::Result<Option<AnyMessageLikeEventContent>> {
        let Some(ref nixpkgs_pr) = self.config.nixpkgs_pr else {
            return Ok(None);
        };

        let client = &self.http;
        let result = fetch_nixpkgs_pr(client, &nixpkgs_pr.token, pr_number).await?;

        if track {
            if !self.room.is_direct().await? {
                return Ok(Some(AnyMessageLikeEventContent::RoomMessage(
                    RoomMessageEventContent::text_plain(
                        "Tracking Nixpkgs PR is only avaliable in a DM!",
                    ),
                )));
            }
            let pr_info = result.clone();
            let config = self.config.clone();
            let http = self.http.clone();
            let room = self.room.clone();
            tokio::spawn(async move {
                use crate::command::functions::nixpkgs_pr::pr_state_stream;

                let config = config;

                let Some(ref nixpkgs_pr) = config.nixpkgs_pr else {
                    return;
                };
                let Some(ref cron) = nixpkgs_pr.cron else {
                    return;
                };
                let token = &nixpkgs_pr.token;

                tokio::time::sleep(Duration::from_secs(1)).await;

                let stream = pr_state_stream(&http, cron, token, pr_number, pr_info);
                pin_mut!(stream);

                tracing::debug!(room_id = %room.room_id(), "Start tracking Nixpkgs PR #{pr_number}");
                while let Some(status) = stream.next().await {
                    use crate::command::functions::nixpkgs_pr::TrackStatus;
                    match status {
                        TrackStatus::Pending { new_branch, .. } => {
                            if let Some(new_branch) = new_branch {
                                use crate::command::functions::nixpkgs_pr::NewBranch;
                                let format_str = match new_branch {
                                    NewBranch::StagingNext => "staging-next",
                                    NewBranch::Master => "master",
                                    NewBranch::UnstableSmall => "nixos-unstable-small",
                                    NewBranch::NixpkgsUnstable => "nixpkgs-unstable",
                                    NewBranch::Unstable => "nixos-unstable",
                                };
                                if let Err(e) = room
                                    .send(RoomMessageEventContent::text_plain(format!(
                                        "PR #{pr_number} is now in branch {format_str}!"
                                    )))
                                    .await
                                {
                                    tracing::warn!("Failed to send status: {e:?}");
                                }
                            }
                        }
                        TrackStatus::Done => {
                            if let Err(e) = room
                                .send(RoomMessageEventContent::text_plain(format!(
                                    "PR #{pr_number} is now in all branches!"
                                )))
                                .await
                            {
                                tracing::warn!("Failed to send status: {e:?}");
                            }
                            return;
                        }
                    }
                }
            });
        }

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
                    "{track_or_not}PR #{pr_number}: {title} https://github.com/NixOS/nixpkgs/pull/{pr_number}{in_branches}",
                    track_or_not = if track { "Tracking " } else { "" },
                    title = result.title,
                    in_branches = in_branches,
                ),
                format!(
                    "<p>{track_or_not}<a href='https://github.com/NixOS/nixpkgs/pull/{pr_number}'>PR #{pr_number}: {title}</a>{in_branches}",
                    track_or_not = if track { "Tracking " } else { "" },
                    title = result.title,
                    in_branches = in_branches_html,
                ),
            ),
        )))
    }
}
