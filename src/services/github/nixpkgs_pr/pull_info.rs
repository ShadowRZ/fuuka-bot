#![allow(clippy::all, warnings)]
use super::GitObjectID;
pub struct PullInfo;
pub mod pull_info {
    #![allow(dead_code)]
    use std::result::Result;
    pub const OPERATION_NAME: &str = "PullInfo";
    pub const QUERY : & str = "query PullInfo($pr_number: Int!) {\n  repository(owner: \"NixOS\", name: \"nixpkgs\") {\n    pullRequest(number: $pr_number) {\n      title\n      state\n      mergeCommit {\n        head: oid\n      }\n    }\n  }\n}\n" ;
    use super::*;
    use serde::{Deserialize, Serialize};
    #[allow(dead_code)]
    type Boolean = bool;
    #[allow(dead_code)]
    type Float = f64;
    #[allow(dead_code)]
    type Int = i64;
    #[allow(dead_code)]
    type ID = String;
    type GitObjectID = super::GitObjectID;
    #[derive(Clone)]
    pub enum PullRequestState {
        CLOSED,
        MERGED,
        OPEN,
        Other(String),
    }
    impl ::serde::Serialize for PullRequestState {
        fn serialize<S: serde::Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
            ser.serialize_str(match *self {
                PullRequestState::CLOSED => "CLOSED",
                PullRequestState::MERGED => "MERGED",
                PullRequestState::OPEN => "OPEN",
                PullRequestState::Other(ref s) => &s,
            })
        }
    }
    impl<'de> ::serde::Deserialize<'de> for PullRequestState {
        fn deserialize<D: ::serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let s: String = ::serde::Deserialize::deserialize(deserializer)?;
            match s.as_str() {
                "CLOSED" => Ok(PullRequestState::CLOSED),
                "MERGED" => Ok(PullRequestState::MERGED),
                "OPEN" => Ok(PullRequestState::OPEN),
                _ => Ok(PullRequestState::Other(s)),
            }
        }
    }
    #[derive(Serialize)]
    pub struct Variables {
        pub pr_number: Int,
    }
    impl Variables {}
    #[derive(Deserialize)]
    pub struct ResponseData {
        pub repository: Option<PullInfoRepository>,
    }
    #[derive(Deserialize)]
    pub struct PullInfoRepository {
        #[serde(rename = "pullRequest")]
        pub pull_request: Option<PullInfoRepositoryPullRequest>,
    }
    #[derive(Deserialize)]
    pub struct PullInfoRepositoryPullRequest {
        pub title: String,
        pub state: PullRequestState,
        #[serde(rename = "mergeCommit")]
        pub merge_commit: Option<PullInfoRepositoryPullRequestMergeCommit>,
    }
    #[derive(Deserialize)]
    pub struct PullInfoRepositoryPullRequestMergeCommit {
        pub head: GitObjectID,
    }
}
impl graphql_client::GraphQLQuery for PullInfo {
    type Variables = pull_info::Variables;
    type ResponseData = pull_info::ResponseData;
    fn build_query(variables: Self::Variables) -> ::graphql_client::QueryBody<Self::Variables> {
        graphql_client::QueryBody {
            variables,
            query: pull_info::QUERY,
            operation_name: pull_info::OPERATION_NAME,
        }
    }
}
