use http_body_util::BodyExt;
use std::{backtrace::Backtrace, sync::Arc, time::Duration};
use tower::BoxError;
use tower_http::map_response_body::MapResponseBodyLayer;

use octocrab::{
    AuthState, Octocrab, OctocrabBuilder,
    models::pulls::PullRequest,
    service::middleware::{
        base_uri::BaseUriLayer,
        cache::{HttpCacheLayer, mem::InMemoryCache},
    },
};
use secrecy::SecretString;

use crate::{config::RepositoryParts, services::github::pr_tracker::streams::CronStream};

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
        .concurrency_limit(1)
        .rate_limit(1, Duration::from_secs(1))
        .layer(BaseUriLayer::new(base_url))
        .layer(HttpCacheLayer::new(Some(Arc::new(InMemoryCache::new()))))
        .layer(MapResponseBodyLayer::new(|resp: reqwest::Body| {
            resp.map_err(|error| octocrab::Error::Service {
                source: Box::new(error) as BoxError,
                backtrace: Backtrace::capture(),
            })
            .boxed()
        }))
        .layer(crate::middleware::reqwest::ReqwestLayer)
        .service(client.clone());
    OctocrabBuilder::new_empty()
        .with_service(service)
        .with_auth(AuthState::AccessToken { token })
        .build()
        .unwrap()
}

pub async fn pull_request(octocrab: &Octocrab, params: Params) -> anyhow::Result<PullRequest> {
    let Params {
        repository,
        pr_number,
    } = params;
    let RepositoryParts { owner, repo } = repository;

    Ok(octocrab.pulls(owner, repo).get(pr_number as u64).await?)
}
