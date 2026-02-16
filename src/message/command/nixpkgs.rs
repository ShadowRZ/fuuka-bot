use octocrab::models::commits::GithubCommitStatus;

use crate::{
    Context,
    config::RepositoryParts,
    services::github::{Params, models::PullRequestState},
};
use matrix_sdk::{
    Room,
    event_handler::Ctx,
    ruma::events::room::message::{
        AddMentions, ForwardThread, OriginalRoomMessageEvent, RoomMessageEventContent,
    },
};

#[tracing::instrument(name = "nixpkgs", skip(ev, room, context), err)]
pub async fn process(
    ev: &OriginalRoomMessageEvent,
    room: &Room,
    context: &Ctx<Context>,
    pr_number: i32,
    track: bool,
) -> anyhow::Result<()> {
    let Ctx(Context { github, .. }) = context;

    let repository = RepositoryParts {
        owner: "NixOS".to_string(),
        repo: "nixpkgs".to_string(),
    };

    let Some(github) = github else {
        return Ok(());
    };
    let result = crate::services::github::pull_request(
        &github.octocrab,
        Params {
            repository: repository.clone(),
            pr_number,
        },
    )
    .await
    .map_err(crate::Error::GitHubError)?;

    // if track {
    //     if !room.is_direct().await? {
    //         room.send(
    //             RoomMessageEventContent::text_plain(
    //                 "Tracking Nixpkgs PR is only avaliable in a DM!",
    //             )
    //             .make_reply_to(ev, ForwardThread::No, AddMentions::Yes),
    //         )
    //         .await?;
    //         return Ok(());
    //     }
    //     let pr_info = result.clone();

    //     let room = room.clone();
    //     let client = http.clone();
    //     tokio::spawn(async move {
    //         use crate::services::github::nixpkgs_pr::track_nixpkgs_pr;

    //         let Some(ref cron) = nixpkgs_pr.cron else {
    //             return;
    //         };
    //         let token = &nixpkgs_pr.token;

    //         tokio::time::sleep(Duration::from_secs(1)).await;

    //         let stream = track_nixpkgs_pr(&client, cron, token, pr_number, pr_info);
    //         pin_mut!(stream);

    //         tracing::debug!("Start tracking Nixpkgs PR #{pr_number}");
    //         while let Some(status) = stream.next().await {
    //             use crate::services::github::nixpkgs_pr::TrackStatus;
    //             match status {
    //                 TrackStatus::Pending { new_branch, .. } => {
    //                     if let Some(new_branch) = new_branch {
    //                         use crate::services::github::nixpkgs_pr::NewBranch;
    //                         let format_str = match new_branch {
    //                             NewBranch::StagingNext => "staging-next",
    //                             NewBranch::Master => "master",
    //                             NewBranch::UnstableSmall => "nixos-unstable-small",
    //                             NewBranch::NixpkgsUnstable => "nixpkgs-unstable",
    //                             NewBranch::Unstable => "nixos-unstable",
    //                         };
    //                         if let Err(e) = room
    //                             .send(RoomMessageEventContent::text_plain(format!(
    //                                 "PR #{pr_number} is now in branch {format_str}!"
    //                             )))
    //                             .await
    //                         {
    //                             tracing::warn!("Failed to send status: {e}");
    //                         }
    //                     }
    //                 }
    //                 TrackStatus::Done => {
    //                     if let Err(e) = room
    //                         .send(RoomMessageEventContent::text_plain(format!(
    //                             "PR #{pr_number} is now in all branches!"
    //                         )))
    //                         .await
    //                     {
    //                         tracing::warn!("Failed to send status: {e}");
    //                     }
    //                     return;
    //                 }
    //             }
    //         }
    //     });
    // }

    let all_branches = github
        .pr_tracker
        .all_branches(&repository, &result.base_ref_name);

    let mut in_branches_data: Vec<(String, bool)> = Vec::new();

    if let PullRequestState::MERGED { merge_commit, .. } = result.state {
        for branch in all_branches {
            let compare = github
                .octocrab
                .commits(&repository.owner, &repository.repo)
                .compare(&branch, &merge_commit.oid)
                .per_page(1)
                .send()
                .await?;
            let in_branch = matches!(
                compare.status,
                GithubCommitStatus::Behind | GithubCommitStatus::Identical
            );
            in_branches_data.push((branch, in_branch));
        }
    };

    let mut in_branches = String::new();

    for (branch, in_branch) in in_branches_data.iter() {
        in_branches.push_str(&format!(
            "\n{branch} {compare}",
            compare = if *in_branch { "✅" } else { "-" }
        ));
    }

    let mut in_branches_html = String::new();

    if !in_branches_data.is_empty() {
        in_branches_html.push_str("<p>");
        for (branch, in_branch) in in_branches_data.iter() {
            in_branches_html.push_str(&format!(
                "<{tag}>{branch}</{tag}>{compare}",
                tag = if *in_branch { "b" } else { "del" },
                compare = if *in_branch { " ✅<br/>" } else { "<br/>" }
            ));
        }
        in_branches_html.push_str("</p>");
    }

    room.send(RoomMessageEventContent::text_html(
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
                    ev,
                    ForwardThread::No,
                    AddMentions::Yes,
                )).await?;

    Ok(())
}
