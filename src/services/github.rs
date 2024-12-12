use std::sync::OnceLock;

static GRAPHQL_ENDPOINT: &str = "https://api.github.com/graphql";
static GQL_CLIENT: OnceLock<gql_client::Client> = OnceLock::new();

pub mod nixpkgs_pr {

    pub mod pull_info {
        use serde::{Deserialize, Serialize};

        #[derive(Deserialize, Clone, Debug)]
        #[serde(rename_all = "camelCase")]
        pub struct PullInfo {
            pub repository: Option<Repository>,
        }

        #[derive(Deserialize, Clone, Debug)]
        #[serde(rename_all = "camelCase")]
        pub struct Repository {
            pub pull_request: Option<PullRequest>,
        }

        #[derive(Deserialize, Clone, Debug)]
        #[serde(rename_all = "camelCase")]
        pub struct PullRequest {
            pub title: String,
            pub state: PullRequestState,
            pub merge_commit: Option<Commit>,
        }

        #[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[serde(rename_all = "UPPERCASE")]
        pub enum PullRequestState {
            Closed,
            Merged,
            Open,
        }

        #[derive(Deserialize, Clone, Debug)]
        #[serde(rename_all = "camelCase")]
        pub struct Commit {
            pub head: GitObjectId,
        }

        #[derive(Deserialize, Clone, Debug)]
        #[serde(transparent)]
        pub struct GitObjectId(pub String);

        #[derive(Serialize, Clone, Debug)]
        pub struct PullInfoVariables {
            pub pr_number: i32,
        }
    }

    pub mod pull_branches {
        use serde::{Deserialize, Serialize};

        #[derive(Deserialize, Clone, Debug)]
        #[serde(rename_all = "camelCase")]
        pub struct PullBranches {
            pub merged_branches: Option<Repository>,
        }

        #[derive(Deserialize, Clone, Debug)]
        #[serde(rename_all = "camelCase")]
        pub struct Repository {
            pub staging: Option<Ref>,
            pub master: Option<Ref>,
            pub nixos_unstable_small: Option<Ref>,
            pub nixpkgs_unstable: Option<Ref>,
            pub nixos_unstable: Option<Ref>,
        }

        #[derive(Deserialize, Clone, Debug)]
        #[serde(rename_all = "camelCase")]
        pub struct Ref {
            pub compare: Option<Comparison>,
        }

        #[derive(Deserialize, Clone, Debug)]
        #[serde(rename_all = "camelCase")]
        pub struct Comparison {
            pub status: ComparisonStatus,
        }

        #[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[serde(rename_all = "UPPERCASE")]
        pub enum ComparisonStatus {
            Ahead,
            Behind,
            Diverged,
            Identical,
        }

        impl ComparisonStatus {
            pub fn is_included(self) -> bool {
                self == ComparisonStatus::Identical || self == ComparisonStatus::Behind
            }
        }

        #[derive(Serialize, Clone, Debug)]
        pub struct PullBranchesVariables<'a> {
            pub head: &'a str,
        }
    }

    use std::pin::Pin;

    use pin_project_lite::pin_project;

    use self::pull_info::PullRequestState;
    use futures_util::{stream::BoxStream, Stream, StreamExt};

    #[derive(Debug, Clone)]
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

    fn get_gql_client(token: &str) -> &gql_client::Client {
        super::GQL_CLIENT.get_or_init(|| {
            use std::collections::HashMap;

            let mut headers = HashMap::new();
            headers.insert("Authorization", format!("Bearer {token}"));
            headers.insert("User-Agent", crate::APP_USER_AGENT.to_string());

            gql_client::Client::new_with_headers(super::GRAPHQL_ENDPOINT, headers)
        })
    }

    pub async fn fetch_nixpkgs_pr(token: &str, pr_number: i32) -> anyhow::Result<PrInfo> {
        let (title, state, head) = fetch_pr_basic_info(token, pr_number).await?;

        let in_branches = match head {
            Some(ref head) => Some(fetch_head_in_branches(token, head).await?),
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
        token: &str,
        pr_number: i32,
    ) -> anyhow::Result<(String, PullRequestState, Option<String>)> {
        use self::pull_info::{PullInfo, PullInfoVariables};

        let vars = PullInfoVariables { pr_number };
        let resp = get_gql_client(token)
            .query_with_vars::<PullInfo, PullInfoVariables>(
                include_str!("queries/pr-info.graphql"),
                vars,
            )
            .await
            .map_err(|e| crate::Error::GraphQLError {
                service: "github",
                error: e,
            })?;
        let Some(data) = resp else {
            return Err(crate::Error::UnexpectedError("Server returned no valid data!").into());
        };

        let Some(repository) = data.repository else {
            return Err(
                crate::Error::UnexpectedError("NixOS/nixpkgs repository disappeared!").into(),
            );
        };

        let Some(pull_request) = repository.pull_request else {
            return Err(crate::Error::UnexpectedError("This PR is not a pull request!").into());
        };

        let title = pull_request.title;
        let head = pull_request.merge_commit.map(|c| c.head.0);
        let state = pull_request.state;

        Ok((title, state, head))
    }

    async fn fetch_head_in_branches(token: &str, head: &str) -> anyhow::Result<PrBranchesStatus> {
        use self::pull_branches::{PullBranches, PullBranchesVariables};

        let vars = PullBranchesVariables { head };
        let resp = get_gql_client(token)
            .query_with_vars::<PullBranches, PullBranchesVariables>(
                include_str!("queries/branches.graphql"),
                vars,
            )
            .await
            .map_err(|e| crate::Error::GraphQLError {
                service: "github",
                error: e,
            })?;
        let Some(data) = resp else {
            return Err(crate::Error::UnexpectedError("Server returned no valid data!").into());
        };
        let Some(merged_branches) = data.merged_branches else {
            anyhow::bail!("NixOS/nixpkgs repository disappeared!");
        };

        Ok(PrBranchesStatus {
            staging: merged_branches
                .staging
                .and_then(|ref_| ref_.compare)
                .map(|compare| compare.status.is_included())
                .unwrap_or_default(),
            master: merged_branches
                .master
                .and_then(|ref_| ref_.compare)
                .map(|compare| compare.status.is_included())
                .unwrap_or_default(),
            nixos_unstable_small: merged_branches
                .nixos_unstable_small
                .and_then(|ref_| ref_.compare)
                .map(|compare| compare.status.is_included())
                .unwrap_or_default(),
            nixpkgs_unstable: merged_branches
                .nixpkgs_unstable
                .and_then(|ref_| ref_.compare)
                .map(|compare| compare.status.is_included())
                .unwrap_or_default(),
            nixos_unstable: merged_branches
                .nixos_unstable
                .and_then(|ref_| ref_.compare)
                .map(|compare| compare.status.is_included())
                .unwrap_or_default(),
        })
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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

    pub fn track_nixpkgs_pr<'a>(
        cron: &'a cronchik::CronSchedule,
        token: &'a str,
        pr_number: i32,
        pr_info: PrInfo,
    ) -> TrackNixpkgsPr<'a> {
        let stream = futures_util::stream::unfold(Some((pr_info, pr_number)), |state| async {
            let (state, pr_number) = state?;
            fetch_next_pr_track_status(cron, token, pr_number, Some(state))
                .await
                .ok()
        })
        .boxed();
        TrackNixpkgsPr { stream }
    }

    async fn fetch_next_pr_track_status<'a>(
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
                PullRequestState::Closed => Ok((TrackStatus::Done, Some((pr_info, pr_number)))),
                PullRequestState::Open => {
                    let next_pr_info = fetch_nixpkgs_pr(token, pr_number).await?;

                    let state = next_pr_info.state;
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
                PullRequestState::Merged => {
                    let state = pr_info.state;
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
                    let next_branches = fetch_head_in_branches(token, head).await?;
                    let new_branch = new_branch_from_status(&next_branches, prev_branches);

                    let mut new_pr_info = pr_info.clone();
                    new_pr_info.in_branches = Some(next_branches);

                    Ok((
                        TrackStatus::Pending { state, new_branch },
                        Some((new_pr_info, pr_number)),
                    ))
                }
            },
            None => {
                let next_pr_info = fetch_nixpkgs_pr(token, pr_number).await?;

                let state = next_pr_info.state;

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
}
