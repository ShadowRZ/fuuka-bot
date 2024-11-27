use std::sync::Arc;

use futures_util::{pin_mut, StreamExt};
use matrix_sdk::ruma::events::room::message::OriginalRoomMessageEvent;
use matrix_sdk::ruma::events::room::message::{AddMentions, ForwardThread};
use matrix_sdk::ruma::events::{
    room::message::RoomMessageEventContent, AnyMessageLikeEventContent,
};
use matrix_sdk::Room;
use std::time::Duration;

use crate::Config;

use super::{Event, OutgoingContent, OutgoingResponse};

pub fn event_handler() -> super::EventHandler {
    dptree::case![Event::Nixpkgs { pr_number, track }].endpoint(
        |(pr_number, track): (i32, bool),
         ev: Arc<OriginalRoomMessageEvent>,
         room: Arc<Room>,
         config: Arc<Config>,
         http: reqwest::Client| async move {
            use crate::command::functions::nixpkgs_pr::fetch_nixpkgs_pr;

            let Some(ref nixpkgs_pr) = config.nixpkgs_pr else {
                return Ok(OutgoingResponse {
                    room,
                    content: OutgoingContent::None,
                });
            };

            let client = &http;
            let result = fetch_nixpkgs_pr(client, &nixpkgs_pr.token, pr_number).await?;

            if track {
                if !room.is_direct().await? {
                    return Ok(
                        OutgoingResponse {
                            room,
                            content: OutgoingContent::Event(AnyMessageLikeEventContent::RoomMessage(
                                RoomMessageEventContent::text_plain(
                                    "Tracking Nixpkgs PR is only avaliable in a DM!",
                                ).make_reply_to(
                                    &ev,
                                    ForwardThread::No,
                                    AddMentions::Yes,
                                ),
                            ))
                        }
                    );
                }
                let pr_info = result.clone();
                let config = config.clone();
                let http = http.clone();
                let room = room.clone();
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

            Ok(OutgoingResponse {
                room,
                content: OutgoingContent::Event(AnyMessageLikeEventContent::RoomMessage(
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
                    ).make_reply_to(
                        &ev,
                        ForwardThread::No,
                        AddMentions::Yes,
                    ),
                ))
            })
         },
    )
}
