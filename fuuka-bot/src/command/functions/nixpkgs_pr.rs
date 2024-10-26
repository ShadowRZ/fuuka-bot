use cynic::{http::ReqwestExt, QueryBuilder};

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
    pr_number: i32,
) -> anyhow::Result<PrInfo> {
    use crate::services::github::pull_info::{PullInfo, PullInfoVariables};

    let operation = PullInfo::build(PullInfoVariables { pr_number });
    let resp = client
        .post(GRAPHQL_ENDPOINT)
        .bearer_auth(token)
        .run_graphql(operation)
        .await?;
    let Some(data) = resp.data else {
        return Err(crate::Error::GraphQLError {
            service: "github",
            errors: resp.errors.unwrap_or_default(),
        }
        .into());
    };

    let Some(repository) = data.repository else {
        anyhow::bail!("NixOS/nixpkgs repository disappeared!");
    };

    let Some(pull_request) = repository.pull_request else {
        return Err(crate::Error::UnexpectedError("This PR is not a pull request!").into());
    };

    let title = pull_request.title;
    //let state = resp.data.repository.pull_request.state;

    let in_branches = match pull_request.merge_commit {
        Some(merge_commit) => {
            use crate::services::github::pull_branches::{PullBranches, PullBranchesVariables};
            use crate::services::github::pull_info::GitObjectId;

            let GitObjectId(head) = merge_commit.head;
            let operation = PullBranches::build(PullBranchesVariables { head });
            let resp = client
                .post(GRAPHQL_ENDPOINT)
                .bearer_auth(token)
                .run_graphql(operation)
                .await?;
            let Some(data) = resp.data else {
                return Err(crate::Error::GraphQLError {
                    service: "github",
                    errors: resp.errors.unwrap_or_default(),
                }
                .into());
            };
            let Some(merged_branches) = data.merged_branches else {
                anyhow::bail!("NixOS/nixpkgs repository disappeared!");
            };

            Some(PrBranchesStatus {
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
        None => None,
    };

    Ok(PrInfo {
        title,
        //state,
        in_branches,
    })
}
