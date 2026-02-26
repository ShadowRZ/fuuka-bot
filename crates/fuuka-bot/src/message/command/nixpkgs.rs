use anyhow::Context as _;
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
    .context(format!(
        "Error while fetching infomation {owner}/{repo}#{pr_number}",
        owner = &repository.owner,
        repo = &repository.repo
    ))?;

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

    if track {
        if !room.is_direct().await? {
            room.send(
                RoomMessageEventContent::text_plain(
                    "Tracking Nixpkgs PR is only avaliable in a DM!",
                )
                .make_reply_to(ev, ForwardThread::No, AddMentions::Yes),
            )
            .await?;
            return Ok(());
        }

        let Some(ref cron) = github.cron else {
            return Ok(());
        };

        let result = loop {
            cron.wait_for_next_tick().await;

            let result = crate::services::github::pull_request(
                &github.octocrab,
                Params {
                    repository: repository.clone(),
                    pr_number,
                },
            )
            .await;

            match result {
                Ok(result) => match result.state {
                    PullRequestState::CLOSED { .. } => {
                        if let Err(error) = room
                            .send_queue()
                            .send(
                                RoomMessageEventContent::text_plain(format!(
                                    "PR #{pr_number} is closed! 😞"
                                ))
                                .into(),
                            )
                            .await
                        {
                            tracing::warn!(
                                room_id = %room.room_id(),
                                "Failed to queue merge info to send: {error}",
                            );
                        }
                        return Ok(());
                    }
                    PullRequestState::MERGED { .. } => {
                        if let Err(error) = room
                            .send_queue()
                            .send(
                                RoomMessageEventContent::text_plain(format!(
                                    "PR #{pr_number} is now merged!"
                                ))
                                .into(),
                            )
                            .await
                        {
                            tracing::warn!(
                                room_id = %room.room_id(),
                                "Failed to queue merge info to send: {error}",
                            );
                        }
                        break result;
                    }
                    _ => {}
                },
                Err(error) => tracing::warn!(
                    "Failed to compare {owner}/{repo}#{pr_number}: {error}",
                    owner = &repository.owner,
                    repo = &repository.repo
                ),
            }
        };

        let client = room.client();
        let room_id = room.room_id().to_owned();
        let base = result.base_ref_name.clone();
        if let PullRequestState::MERGED {
            ref merge_commit, ..
        } = result.state
        {
            let head = merge_commit.oid.clone();
            let github = github.clone();
            let repository = repository.clone();
            tokio::spawn(async move {
                crate::services::github::pr_tracker::track(
                    client, github, repository, room_id, pr_number, base, head,
                )
                .await
            });
        }
    }

    Ok(())
}
