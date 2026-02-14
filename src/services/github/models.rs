use serde::{Deserialize, Serialize};
use url::Url;

type Int = i32;
type DateTime = time::OffsetDateTime;
type GitObjectID = String;

#[derive(Deserialize, Debug, Clone)]
pub struct PartialActor {
    pub login: String,
    pub url: Url,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct PartialCommit {
    pub oid: GitObjectID,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "state", rename_all_fields = "camelCase")]
pub enum PullRequestState {
    CLOSED {
        #[serde(deserialize_with = "time::serde::iso8601::deserialize")]
        closed_at: DateTime,
    },
    MERGED {
        #[serde(deserialize_with = "time::serde::iso8601::deserialize")]
        merged_at: DateTime,
        merge_commit: PartialCommit,
    },
    OPEN,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PartialPullRequest {
    pub number: Int,
    pub author: PartialActor,
    pub title: String,
    #[serde(deserialize_with = "time::serde::iso8601::deserialize")]
    pub created_at: DateTime,
    #[serde(
        default,
        deserialize_with = "time::serde::iso8601::option::deserialize"
    )]
    pub last_edited_at: Option<DateTime>,
    #[serde(flatten)]
    pub state: PullRequestState,
    pub locked: bool,
    pub is_draft: bool,
    pub base_ref_name: String,
    pub permalink: Url,
    pub checks_url: Url,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PartialRepository {
    pub pull_request: Option<PartialPullRequest>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PullInfo {
    pub repository: Option<PartialRepository>,
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct PullInfoVariables {
    pub owner: String,
    pub name: String,
    pub pr_number: Int,
}

impl graphql_client::GraphQLQuery for PullInfo {
    type Variables = PullInfoVariables;

    type ResponseData = PullInfo;

    fn build_query(variables: Self::Variables) -> graphql_client::QueryBody<Self::Variables> {
        graphql_client::QueryBody {
            variables,
            query: include_str!("graphql/pull_info.graphql"),
            operation_name: "PullInfo",
        }
    }
}
