use std::{sync::Arc, time::Duration};

use graphql_client::GraphQLQuery;
use octocrab::{
    AuthState, Octocrab, OctocrabBuilder,
    service::middleware::{base_uri::BaseUriLayer, cache::mem::InMemoryCache},
};
use secrecy::SecretString;

use crate::{
    config::RepositoryParts,
    middleware::cache::HttpCacheLayer,
    services::github::{
        models::{PartialPullRequest, PullInfo, PullInfoVariables},
        pr_tracker::streams::CronStream,
    },
};

pub mod models;
pub mod pr_tracker;

#[derive(Clone)]
pub struct Context {
    pub octocrab: Octocrab,
    pub cron: Option<Arc<CronStream>>,
    pub pr_tracker: Arc<pr_tracker::PrTrackerContext>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Params {
    pub repository: RepositoryParts,
    pub pr_number: i32,
}

pub fn octocrab(client: &reqwest::Client, base_url: http::Uri, token: SecretString) -> Octocrab {
    let service = tower::ServiceBuilder::new()
        .buffer(1024)
        .concurrency_limit(1)
        .rate_limit(1, Duration::from_secs(1))
        .layer(BaseUriLayer::new(base_url))
        .layer(HttpCacheLayer::new(Some(Arc::new(InMemoryCache::new()))))
        .layer(crate::middleware::reqwest::ReqwestLayer)
        .service(client.clone());
    OctocrabBuilder::new_empty()
        .with_service(service)
        .with_auth(AuthState::AccessToken { token })
        .build()
        .unwrap()
}

pub async fn pull_request(
    octocrab: &Octocrab,
    params: Params,
) -> anyhow::Result<PartialPullRequest> {
    let Params {
        repository,
        pr_number,
    } = params;
    let RepositoryParts { owner, repo } = repository;

    let resp = octocrab
        .graphql::<graphql_client::Response<PullInfo>>(&PullInfo::build_query(PullInfoVariables {
            owner,
            name: repo,
            pr_number,
        }))
        .await?;

    if let Some(errors) = resp.errors {
        return Err(Error(errors).into());
    }

    let Some(data) = resp.data else {
        anyhow::bail!("Server returned no valid data!")
    };

    let Some(repository) = data.repository else {
        use graphql_client::PathFragment;
        return Err(Error(vec![graphql_client::Error {
            message: "Could not resolve to a Repository with the name NixOS/nixpkgs.".to_string(),
            locations: None,
            path: Some(vec![PathFragment::Key("repository".to_string())]),
            extensions: None,
        }])
        .into());
    };

    let Some(pull_request) = repository.pull_request else {
        use graphql_client::PathFragment;
        return Err(
            // GraphQL: Could not resolve to a PullRequest with the number of ${pr_number}. (repository.pullRequest)
            Error(vec![graphql_client::Error {
                message: format!(
                    "Could not resolve to a PullRequest with the number of {pr_number}."
                ),
                locations: None,
                path: Some(vec![
                    PathFragment::Key("repository".to_string()),
                    PathFragment::Key("pullRequest".to_string()),
                ]),
                extensions: None,
            }])
            .into(),
        );
    };

    Ok(pull_request)
}

#[derive(Debug)]
pub struct Error(Vec<graphql_client::Error>);

// Based on https://github.com/cli/go-gh/blob/e1048dfe671b9aee9367a5e3e720831f4d64b33a/pkg/api/errors.go#L58-L69
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GraphQL: ")?;
        let Self(error) = self;
        let len = error.len();
        for (i, e) in error.iter().enumerate() {
            write!(f, "{}", e.message)?;
            if let Some(paths) = &e.path {
                write!(f, " (")?;
                for (i, path) in paths.iter().enumerate() {
                    use graphql_client::PathFragment;
                    if i > 0 {
                        write!(f, ".")?;
                    }
                    match path {
                        PathFragment::Key(k) => {
                            write!(f, "{}", k)?;
                        }
                        PathFragment::Index(idx) => {
                            write!(f, "[{}]", idx)?;
                        }
                    }
                }
                write!(f, ")")?;
            }
            if i < len - 1 {
                write!(f, ", ")?;
            }
        }

        Ok(())
    }
}

impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
    #[test]
    pub fn test_github_graphql_error_format() {
        use pretty_assertions::assert_eq;

        let error: Vec<graphql_client::Error> = serde_json::from_value(serde_json::json!(
            [
                { "message": "OH NO" },
                { "message": "this is fine" },
            ]
        ))
        .unwrap();

        let error = super::Error(error);

        let result = format!("{error}");

        assert_eq!(result, "GraphQL: OH NO, this is fine");
    }
    #[test]
    pub fn test_github_graphql_error_format_with_path() {
        use pretty_assertions::assert_eq;

        let error: Vec<graphql_client::Error> = serde_json::from_value(serde_json::json!(
            [
                {
                    "message": "OH NO",
                    "path": ["repository", "issue"]
                },
                {
                    "message": "this is fine",
                    "path": ["repository", "issues", 0, "comments"]
                }
            ]
        ))
        .unwrap();

        let error = super::Error(error);

        let result = format!("{error}");

        assert_eq!(
            result,
            "GraphQL: OH NO (repository.issue), this is fine (repository.issues.[0].comments)"
        );
    }
}
