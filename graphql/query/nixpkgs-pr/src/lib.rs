pub mod pull_info {
    use cynic;
    use fuuka_bot_github_schema::schema;

    #[derive(cynic::QueryVariables, Debug)]
    pub struct PullInfoVariables {
        pub pr_number: i32,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(
        schema = "github",
        graphql_type = "Query",
        variables = "PullInfoVariables"
    )]
    pub struct PullInfo {
        #[arguments(owner: "NixOS", name: "nixpkgs")]
        pub repository: Option<Repository>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(schema = "github", variables = "PullInfoVariables")]
    pub struct Repository {
        #[arguments(number: $pr_number)]
        pub pull_request: Option<PullRequest>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(schema = "github")]
    pub struct PullRequest {
        pub title: String,
        pub state: PullRequestState,
        pub merge_commit: Option<Commit>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(schema = "github")]
    pub struct Commit {
        #[cynic(rename = "oid")]
        pub head: GitObjectId,
    }

    #[derive(cynic::Enum, Clone, Copy, Debug)]
    #[cynic(schema = "github")]
    pub enum PullRequestState {
        Closed,
        Merged,
        Open,
    }

    #[derive(cynic::Scalar, Debug, Clone)]
    #[cynic(graphql_type = "GitObjectID")]
    pub struct GitObjectId(pub String);
}

pub mod pull_branches {
    use cynic;
    use fuuka_bot_github_schema::schema;

    #[derive(cynic::QueryVariables, Debug)]
    pub struct PullBranchesVariables {
        pub head: String,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(
        schema = "github",
        graphql_type = "Query",
        variables = "PullBranchesVariables"
    )]
    pub struct PullBranches {
        #[arguments(owner: "NixOS", name: "nixpkgs")]
        #[cynic(rename = "repository")]
        pub merged_branches: Option<Repository>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(schema = "github", variables = "PullBranchesVariables")]
    pub struct Repository {
        #[arguments(qualifiedName: "staging-next")]
        #[cynic(rename = "ref")]
        pub staging: Option<Ref>,
        #[arguments(qualifiedName: "master")]
        #[cynic(rename = "ref")]
        pub master: Option<Ref>,
        #[arguments(qualifiedName: "nixos-unstable-small")]
        #[cynic(rename = "ref")]
        pub nixos_unstable_small: Option<Ref>,
        #[arguments(qualifiedName: "nixpkgs-unstable")]
        #[cynic(rename = "ref")]
        pub nixpkgs_unstable: Option<Ref>,
        #[arguments(qualifiedName: "nixos-unstable")]
        #[cynic(rename = "ref")]
        pub nixos_unstable: Option<Ref>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(schema = "github", variables = "PullBranchesVariables")]
    pub struct Ref {
        #[arguments(headRef: $head)]
        pub compare: Option<Comparison>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(schema = "github")]
    pub struct Comparison {
        pub status: ComparisonStatus,
    }

    #[derive(cynic::Enum, Clone, Copy, Debug, Hash, Eq, PartialEq)]
    #[cynic(schema = "github")]
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
