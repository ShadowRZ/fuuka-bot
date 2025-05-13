mod pull_branches;
mod pull_info;

use std::pin::Pin;

use futures_util::{Stream, StreamExt, stream::BoxStream};
use pin_project_lite::pin_project;
use serde::Deserialize;

use self::pull_info::pull_info::PullRequestState;

#[derive(Deserialize, Clone, Debug)]
#[serde(transparent)]
pub struct GitObjectID(pub String);

#[derive(Clone)]
pub struct PrInfo {
    pub title: String,
    pub state: PullRequestState,
    pub head: Option<String>,
    pub in_branches: Option<PrBranchesStatus>,
}

#[derive(Debug, Clone, Default)]
pub struct PrBranchesStatus {
    pub staging: bool,
    pub master: bool,
    pub nixos_unstable_small: bool,
    pub nixpkgs_unstable: bool,
    pub nixos_unstable: bool,
}

#[tracing::instrument(skip(client, token))]
pub async fn fetch_nixpkgs_pr(
    client: &reqwest::Client,
    token: &str,
    pr_number: i32,
) -> anyhow::Result<PrInfo> {
    let (title, state, head) = fetch_pr_basic_info(client, token, pr_number).await?;

    let in_branches = match head {
        Some(ref head) => Some(fetch_head_in_branches(client, token, head).await?),
        None => None,
    };

    Ok(PrInfo {
        title,
        head,
        state,
        in_branches,
    })
}

async fn fetch_pr_basic_info(
    client: &reqwest::Client,
    token: &str,
    pr_number: i32,
) -> anyhow::Result<(String, PullRequestState, Option<String>)> {
    use self::pull_info::{PullInfo, pull_info::Variables as PullInfoVariables};

    let vars = PullInfoVariables {
        pr_number: pr_number.into(),
    };
    let resp = super::post_github_graphql::<PullInfo>(client, token, vars).await?;
    if let Some(errors) = resp.errors {
        return Err(crate::Error::GraphQLError {
            service: "github",
            error: errors,
        }
        .into());
    }
    let Some(data) = resp.data else {
        return Err(crate::Error::UnexpectedError("Server returned no valid data!").into());
    };

    let Some(repository) = data.repository else {
        return Err(crate::Error::UnexpectedError("NixOS/nixpkgs repository disappeared!").into());
    };

    let Some(pull_request) = repository.pull_request else {
        return Err(crate::Error::UnexpectedError("This PR is not a pull request!").into());
    };

    let title = pull_request.title;
    let head = pull_request.merge_commit.map(|c| c.head.0);
    let state = pull_request.state;

    Ok((title, state, head))
}

async fn fetch_head_in_branches(
    client: &reqwest::Client,
    token: &str,
    head: &str,
) -> anyhow::Result<PrBranchesStatus> {
    use self::pull_branches::{PullBranches, pull_branches::Variables as PullBranchesVariables};

    let vars = PullBranchesVariables {
        head: head.to_string(),
    };
    let resp = super::post_github_graphql::<PullBranches>(client, token, vars).await?;
    if let Some(errors) = resp.errors {
        return Err(crate::Error::GraphQLError {
            service: "github",
            error: errors,
        }
        .into());
    }
    let Some(data) = resp.data else {
        return Err(crate::Error::UnexpectedError("Server returned no valid data!").into());
    };
    let Some(merged_branches) = data.merged_branches else {
        anyhow::bail!("NixOS/nixpkgs repository disappeared!");
    };

    Ok(PrBranchesStatus {
        staging: merged_branches
            .staging
            .and_then(|ref_| ref_.compare)
            .map(|compare| compare.status.included())
            .unwrap_or_default(),
        master: merged_branches
            .master
            .and_then(|ref_| ref_.compare)
            .map(|compare| compare.status.included())
            .unwrap_or_default(),
        nixos_unstable_small: merged_branches
            .nixos_unstable_small
            .and_then(|ref_| ref_.compare)
            .map(|compare| compare.status.included())
            .unwrap_or_default(),
        nixpkgs_unstable: merged_branches
            .nixpkgs_unstable
            .and_then(|ref_| ref_.compare)
            .map(|compare| compare.status.included())
            .unwrap_or_default(),
        nixos_unstable: merged_branches
            .nixos_unstable
            .and_then(|ref_| ref_.compare)
            .map(|compare| compare.status.included())
            .unwrap_or_default(),
    })
}

#[derive(Clone)]
pub enum TrackStatus {
    Pending {
        state: PullRequestState,
        new_branch: Option<NewBranch>,
    },
    Done,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum NewBranch {
    StagingNext,
    Master,
    UnstableSmall,
    NixpkgsUnstable,
    Unstable,
}

pin_project! {
    /// A named stream to emit Nixpkgs PR status.
    pub struct TrackNixpkgsPr<'a> {
        stream: BoxStream<'a, TrackStatus>,
    }
}

impl Stream for TrackNixpkgsPr<'_> {
    type Item = TrackStatus;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let this = self.project();
        this.stream.as_mut().poll_next(cx)
    }
}

#[tracing::instrument(skip(client, cron, token, pr_info))]
pub fn track_nixpkgs_pr<'a>(
    client: &'a reqwest::Client,
    cron: &'a cronchik::CronSchedule,
    token: &'a str,
    pr_number: i32,
    pr_info: PrInfo,
) -> TrackNixpkgsPr<'a> {
    let stream = futures_util::stream::unfold(Some((pr_info, pr_number)), |state| async {
        let (state, pr_number) = state?;
        fetch_next_pr_track_status(client, cron, token, pr_number, Some(state))
            .await
            .ok()
    })
    .boxed();
    TrackNixpkgsPr { stream }
}

async fn fetch_next_pr_track_status<'a>(
    client: &reqwest::Client,
    cron: &'a cronchik::CronSchedule,
    token: &'a str,
    pr_number: i32,
    pr_info: Option<PrInfo>,
) -> anyhow::Result<(TrackStatus, Option<(PrInfo, i32)>)> {
    use time::OffsetDateTime;

    let now = OffsetDateTime::now_utc();
    let next = cron.next_time_from(now);
    tracing::debug!(
        "PR #{pr_number} next tracking time: {next}",
        pr_number = pr_number
    );
    let elpased = next - now;

    tokio::time::sleep(elpased.unsigned_abs()).await;

    match pr_info {
        Some(pr_info) => match pr_info.state {
            PullRequestState::CLOSED => Ok((TrackStatus::Done, Some((pr_info, pr_number)))),
            PullRequestState::OPEN => {
                let next_pr_info = fetch_nixpkgs_pr(client, token, pr_number).await?;

                let state = next_pr_info.state.clone();
                let prev_branches = match &pr_info.in_branches {
                    Some(branches) => branches,
                    None => &PrBranchesStatus::default(),
                };
                let new_branch = next_pr_info
                    .in_branches
                    .as_ref()
                    .and_then(|s| new_branch_from_status(s, prev_branches));

                Ok((
                    TrackStatus::Pending { state, new_branch },
                    Some((next_pr_info, pr_number)),
                ))
            }
            PullRequestState::MERGED => {
                let state = pr_info.state.clone();
                let Some(ref head) = pr_info.head else {
                    return Ok((
                        TrackStatus::Pending {
                            state,
                            new_branch: None,
                        },
                        Some((pr_info, pr_number)),
                    ));
                };
                let prev_branches = match &pr_info.in_branches {
                    Some(branches) => branches,
                    None => &PrBranchesStatus::default(),
                };
                let next_branches = fetch_head_in_branches(client, token, head).await?;
                let new_branch = new_branch_from_status(&next_branches, prev_branches);

                let mut new_pr_info = pr_info.clone();
                new_pr_info.in_branches = Some(next_branches);

                Ok((
                    TrackStatus::Pending { state, new_branch },
                    Some((new_pr_info, pr_number)),
                ))
            }
            PullRequestState::Other(_) => unreachable!(),
        },
        None => {
            let next_pr_info = fetch_nixpkgs_pr(client, token, pr_number).await?;

            let state = next_pr_info.state.clone();

            let new_branch = next_pr_info
                .in_branches
                .as_ref()
                .and_then(|s| new_branch_from_status(s, &PrBranchesStatus::default()));

            Ok((
                TrackStatus::Pending { state, new_branch },
                Some((next_pr_info, pr_number)),
            ))
        }
    }
}

fn new_branch_from_status(
    branches: &PrBranchesStatus,
    prev_branches: &PrBranchesStatus,
) -> Option<NewBranch> {
    if branches.nixos_unstable {
        if !prev_branches.nixos_unstable {
            Some(NewBranch::Unstable)
        } else {
            None
        }
    } else if branches.nixpkgs_unstable {
        if !prev_branches.nixpkgs_unstable {
            Some(NewBranch::NixpkgsUnstable)
        } else {
            None
        }
    } else if branches.nixos_unstable_small {
        if !prev_branches.nixos_unstable_small {
            Some(NewBranch::UnstableSmall)
        } else {
            None
        }
    } else if branches.master {
        if !prev_branches.master {
            Some(NewBranch::Master)
        } else {
            None
        }
    } else if branches.staging {
        if !prev_branches.staging {
            Some(NewBranch::StagingNext)
        } else {
            None
        }
    } else {
        None
    }
}

impl self::pull_branches::pull_branches::ComparisonStatus {
    pub fn included(self) -> bool {
        use self::pull_branches::pull_branches::ComparisonStatus;
        self == ComparisonStatus::IDENTICAL || self == ComparisonStatus::BEHIND
    }
}

mod tests {
    #[test]
    pub fn test_new_branch() {
        use super::{NewBranch, PrBranchesStatus};

        let prev = PrBranchesStatus {
            staging: true,
            master: true,
            nixos_unstable_small: true,
            nixpkgs_unstable: true,
            nixos_unstable: false,
        };
        let next = PrBranchesStatus {
            staging: true,
            master: true,
            nixos_unstable_small: true,
            nixpkgs_unstable: true,
            nixos_unstable: true,
        };

        let result = super::new_branch_from_status(&next, &prev);
        assert_eq!(result, Some(NewBranch::Unstable));
    }

    #[test]
    pub fn test_branch_unchanged() {
        use super::PrBranchesStatus;

        let prev = PrBranchesStatus {
            staging: true,
            master: true,
            nixos_unstable_small: true,
            nixpkgs_unstable: true,
            nixos_unstable: false,
        };
        let next = PrBranchesStatus {
            staging: true,
            master: true,
            nixos_unstable_small: true,
            nixpkgs_unstable: true,
            nixos_unstable: false,
        };

        let result = super::new_branch_from_status(&next, &prev);
        assert_eq!(result, None);
    }
}
