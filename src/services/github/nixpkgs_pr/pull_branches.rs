#![allow(clippy::all, warnings)]
pub struct PullBranches;
pub mod pull_branches {
    #![allow(dead_code)]
    use std::result::Result;
    pub const OPERATION_NAME: &str = "PullBranches";
    pub const QUERY: &str = "query PullBranches($head: String!) {\n  mergedBranches: repository(owner: \"NixOS\", name: \"nixpkgs\") {\n    staging: ref(qualifiedName: \"staging-next\") {\n      compare(headRef: $head) {\n        status\n      }\n    }\n    master: ref(qualifiedName: \"master\") {\n      compare(headRef: $head) {\n        status\n      }\n    }\n    nixosUnstableSmall: ref(qualifiedName: \"nixos-unstable-small\") {\n      compare(headRef: $head) {\n        status\n      }\n    }\n    nixpkgsUnstable: ref(qualifiedName: \"nixpkgs-unstable\") {\n      compare(headRef: $head) {\n        status\n      }\n    }\n    nixosUnstable: ref(qualifiedName: \"nixos-unstable\") {\n      compare(headRef: $head) {\n        status\n      }\n    }\n  }\n}\n";
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
    #[derive(PartialEq, Eq, PartialOrd, Ord)]
    pub enum ComparisonStatus {
        AHEAD,
        BEHIND,
        DIVERGED,
        IDENTICAL,
        Other(String),
    }
    impl ::serde::Serialize for ComparisonStatus {
        fn serialize<S: serde::Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
            ser.serialize_str(match *self {
                ComparisonStatus::AHEAD => "AHEAD",
                ComparisonStatus::BEHIND => "BEHIND",
                ComparisonStatus::DIVERGED => "DIVERGED",
                ComparisonStatus::IDENTICAL => "IDENTICAL",
                ComparisonStatus::Other(ref s) => &s,
            })
        }
    }
    impl<'de> ::serde::Deserialize<'de> for ComparisonStatus {
        fn deserialize<D: ::serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let s: String = ::serde::Deserialize::deserialize(deserializer)?;
            match s.as_str() {
                "AHEAD" => Ok(ComparisonStatus::AHEAD),
                "BEHIND" => Ok(ComparisonStatus::BEHIND),
                "DIVERGED" => Ok(ComparisonStatus::DIVERGED),
                "IDENTICAL" => Ok(ComparisonStatus::IDENTICAL),
                _ => Ok(ComparisonStatus::Other(s)),
            }
        }
    }
    #[derive(Serialize)]
    pub struct Variables {
        pub head: String,
    }
    impl Variables {}
    #[derive(Deserialize)]
    pub struct ResponseData {
        #[serde(rename = "mergedBranches")]
        pub merged_branches: Option<PullBranchesMergedBranches>,
    }
    #[derive(Deserialize)]
    pub struct PullBranchesMergedBranches {
        pub staging: Option<PullBranchesMergedBranchesStaging>,
        pub master: Option<PullBranchesMergedBranchesMaster>,
        #[serde(rename = "nixosUnstableSmall")]
        pub nixos_unstable_small: Option<PullBranchesMergedBranchesNixosUnstableSmall>,
        #[serde(rename = "nixpkgsUnstable")]
        pub nixpkgs_unstable: Option<PullBranchesMergedBranchesNixpkgsUnstable>,
        #[serde(rename = "nixosUnstable")]
        pub nixos_unstable: Option<PullBranchesMergedBranchesNixosUnstable>,
    }
    #[derive(Deserialize)]
    pub struct PullBranchesMergedBranchesStaging {
        pub compare: Option<PullBranchesMergedBranchesStagingCompare>,
    }
    #[derive(Deserialize)]
    pub struct PullBranchesMergedBranchesStagingCompare {
        pub status: ComparisonStatus,
    }
    #[derive(Deserialize)]
    pub struct PullBranchesMergedBranchesMaster {
        pub compare: Option<PullBranchesMergedBranchesMasterCompare>,
    }
    #[derive(Deserialize)]
    pub struct PullBranchesMergedBranchesMasterCompare {
        pub status: ComparisonStatus,
    }
    #[derive(Deserialize)]
    pub struct PullBranchesMergedBranchesNixosUnstableSmall {
        pub compare: Option<PullBranchesMergedBranchesNixosUnstableSmallCompare>,
    }
    #[derive(Deserialize)]
    pub struct PullBranchesMergedBranchesNixosUnstableSmallCompare {
        pub status: ComparisonStatus,
    }
    #[derive(Deserialize)]
    pub struct PullBranchesMergedBranchesNixpkgsUnstable {
        pub compare: Option<PullBranchesMergedBranchesNixpkgsUnstableCompare>,
    }
    #[derive(Deserialize)]
    pub struct PullBranchesMergedBranchesNixpkgsUnstableCompare {
        pub status: ComparisonStatus,
    }
    #[derive(Deserialize)]
    pub struct PullBranchesMergedBranchesNixosUnstable {
        pub compare: Option<PullBranchesMergedBranchesNixosUnstableCompare>,
    }
    #[derive(Deserialize)]
    pub struct PullBranchesMergedBranchesNixosUnstableCompare {
        pub status: ComparisonStatus,
    }
}
impl graphql_client::GraphQLQuery for PullBranches {
    type Variables = pull_branches::Variables;
    type ResponseData = pull_branches::ResponseData;
    fn build_query(variables: Self::Variables) -> ::graphql_client::QueryBody<Self::Variables> {
        graphql_client::QueryBody {
            variables,
            query: pull_branches::QUERY,
            operation_name: pull_branches::OPERATION_NAME,
        }
    }
}
