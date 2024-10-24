mod models {
    pub mod pr_info {
        use serde::Deserialize;
        use serde::Serialize;

        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        #[serde(rename_all = "camelCase")]
        pub struct PrInfo {
            pub data: PrInfoData,
        }

        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        #[serde(rename_all = "camelCase")]
        pub struct PrInfoData {
            pub repository: PrInfoRepository,
        }

        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        #[serde(rename_all = "camelCase")]
        pub struct PrInfoRepository {
            pub pull_request: PrInfoPullRequest,
        }

        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        #[serde(rename_all = "camelCase")]
        pub struct PrInfoPullRequest {
            pub title: String,
            pub state: PullRequestState,
            pub merge_commit: Option<PrInfoMergeCommit>,
        }

        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        #[serde(rename_all = "camelCase")]
        pub struct PrInfoMergeCommit {
            pub head: String,
        }

        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        #[serde(rename_all = "UPPERCASE")]
        pub enum PullRequestState {
            Closed,
            Merged,
            Open,
        }
    }

    pub mod pr_branches {
        use serde::Deserialize;
        use serde::Serialize;

        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        #[serde(rename_all = "camelCase")]
        pub struct PrBranches {
            pub data: PrBranchesData,
        }

        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        #[serde(rename_all = "camelCase")]
        pub struct PrBranchesData {
            pub merged_branches: PrBranchesMergedBranches,
        }

        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        #[serde(rename_all = "camelCase")]
        pub struct PrBranchesMergedBranches {
            pub staging: Comparison,
            pub master: Comparison,
            pub nixos_unstable_small: Comparison,
            pub nixpkgs_unstable: Comparison,
            pub nixos_unstable: Comparison,
        }

        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        pub struct Comparison {
            pub compare: ComparisonStatusWrapper,
        }

        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        pub struct ComparisonStatusWrapper {
            pub status: ComparisonStatus,
        }

        #[derive(Copy, Clone, Debug, Deserialize, Serialize, Hash, Eq, PartialEq)]
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
    }
}

mod requests {
    use serde::Deserialize;
    use serde::Serialize;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct PrInfoRequest {
        query: &'static str,
        variables: PrInfoRequestVariables,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct PrInfoRequestVariables {
        pub pr_number: u64,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct PrBranchesRequest {
        query: &'static str,
        variables: PrBranchesRequestVariables,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct PrBranchesRequestVariables {
        pub head: String,
    }

    impl PrInfoRequest {
        pub fn new(pr_number: u64) -> PrInfoRequest {
            PrInfoRequest {
                query: include_str!("./graphql/pr-info.graphql"),
                variables: PrInfoRequestVariables { pr_number },
            }
        }
    }

    impl PrBranchesRequest {
        pub fn new(head: String) -> PrBranchesRequest {
            PrBranchesRequest {
                query: include_str!("./graphql/branches.graphql"),
                variables: PrBranchesRequestVariables { head },
            }
        }
    }
}

static GRAPHQL_ENDPOINT: &str = "https://api.github.com/graphql";

pub struct PrInfo {
    pub title: String,
    //pub state: self::models::pr_info::PullRequestState,
    pub in_branches: Option<PrBranchesStatus>,
}

pub struct PrBranchesStatus {
    pub staging: bool,
    pub master: bool,
    pub nixos_unstable_small: bool,
    pub nixpkgs_unstable: bool,
    pub nixos_unstable: bool,
}

pub async fn fetch_nixpkgs_pr(
    client: &reqwest::Client,
    token: &str,
    pr_number: u64,
) -> anyhow::Result<PrInfo> {
    use self::requests::PrInfoRequest;

    let body = PrInfoRequest::new(pr_number);
    let resp = client
        .post(GRAPHQL_ENDPOINT)
        .bearer_auth(token)
        .json(&body)
        .send()
        .await?
        .error_for_status()?
        .json::<self::models::pr_info::PrInfo>()
        .await?;

    let title = resp.data.repository.pull_request.title;
    //let state = resp.data.repository.pull_request.state;

    let in_branches = match resp.data.repository.pull_request.merge_commit {
        Some(merge_commit) => {
            use self::requests::PrBranchesRequest;

            let head = merge_commit.head;
            let body = PrBranchesRequest::new(head);
            let resp = client
                .post(GRAPHQL_ENDPOINT)
                .bearer_auth(token)
                .json(&body)
                .send()
                .await?
                .error_for_status()?
                .json::<self::models::pr_branches::PrBranches>()
                .await?;

            Some(PrBranchesStatus {
                staging: resp
                    .data
                    .merged_branches
                    .staging
                    .compare
                    .status
                    .is_included(),
                master: resp
                    .data
                    .merged_branches
                    .master
                    .compare
                    .status
                    .is_included(),
                nixos_unstable_small: resp
                    .data
                    .merged_branches
                    .nixos_unstable_small
                    .compare
                    .status
                    .is_included(),
                nixpkgs_unstable: resp
                    .data
                    .merged_branches
                    .nixpkgs_unstable
                    .compare
                    .status
                    .is_included(),
                nixos_unstable: resp
                    .data
                    .merged_branches
                    .nixos_unstable
                    .compare
                    .status
                    .is_included(),
            })
        }
        None => None,
    };

    Ok(PrInfo {
        title,
        //state,
        in_branches,
    })
}
