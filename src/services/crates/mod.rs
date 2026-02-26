use std::time::Duration;

use bytes::Bytes;
use http_body_util::BodyExt;
use serde::Deserialize;
use tower::Service;
use tower::{BoxError, ServiceExt, buffer::Buffer, util::BoxService};
use tower_http::ServiceBuilderExt;

type BoxBody = http_body_util::combinators::BoxBody<Bytes, Error>;

pub type CratesService = Buffer<
    http::Request<BoxBody>,
    <BoxService<http::Request<BoxBody>, http::Response<BoxBody>, BoxError> as tower::Service<
        http::Request<BoxBody>,
    >>::Future,
>;

#[derive(Deserialize, Debug, Clone)]
#[allow(missing_docs)]
pub struct CrateMetadata {
    #[serde(rename = "crate")]
    pub crate_info: CrateInfo,
    pub versions: Option<Vec<CrateVersion>>,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(missing_docs)]
pub struct CrateInfo {
    pub description: Option<String>,
    pub name: String,
    pub max_stable_version: String,
    pub default_version: Option<String>,
    pub downloads: u64,
    pub documentation: Option<String>,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub num_versions: u32,
    pub recent_downloads: Option<u64>,
    pub yanked: bool,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(missing_docs)]
pub struct CrateVersion {
    #[serde(rename = "crate")]
    pub crate_name: String,
    pub description: Option<String>,
    pub documentation: Option<String>,
    pub downloads: u32,
    pub edition: Option<String>,
    pub homepage: Option<String>,
    pub license: Option<String>,
    pub num: String,
    pub rust_version: Option<String>,
    pub yanked: bool,
    pub yank_message: Option<String>,
    pub repository: Option<String>,
}

#[derive(Clone)]
pub struct CratesClient {
    base_url: http::Uri,
    service: CratesService,
}

impl CratesClient {
    pub fn from_reqwest_client(client: &reqwest::Client, base_url: http::Uri) -> Self {
        let service = tower::ServiceBuilder::new()
            .concurrency_limit(1)
            .rate_limit(1, Duration::from_secs(1))
            .map_response_body(|resp: reqwest::Body| {
                resp.map_err(|error| Error {
                    inner: Kind::Other(error.into()),
                })
                .boxed()
            })
            .layer(crate::middleware::reqwest::ReqwestLayer)
            .service(client.clone());

        Self::new(service, base_url)
    }

    pub fn new<S>(service: S, base_url: http::Uri) -> Self
    where
        S: tower::Service<http::Request<BoxBody>, Response = http::Response<BoxBody>>
            + Send
            + 'static,
        S::Future: Send + 'static,
        S::Error: Into<BoxError>,
    {
        let service = Buffer::new(BoxService::new(service.map_err(Into::into)), 1024);

        Self { service, base_url }
    }

    pub async fn crate_info(&self, name: String) -> Result<CrateMetadata, Error> {
        let builder = http::uri::Builder::from(self.base_url.clone());
        let uri = builder
            .path_and_query(format!("/api/v1/crates/{name}"))
            .build()
            .map_err(|e| Error {
                inner: Kind::Http(e),
            })?;

        let request = http::Request::builder()
            .method("GET")
            .uri(uri)
            .body(
                http_body_util::Empty::<Bytes>::new()
                    .map_err(|e| Error {
                        inner: Kind::Other(Box::new(e) as BoxError),
                    })
                    .boxed(),
            )
            .map_err(|e| Error {
                inner: Kind::Http(e),
            })?;
        let mut service = self.service.clone();
        let response = service
            .ready()
            .await
            .map_err(|e| Error {
                inner: Kind::Service(e),
            })?
            .call(request)
            .await
            .map_err(|e| Error {
                inner: Kind::Service(e),
            })?;

        let body = response.into_body();
        let bytes = body.collect().await?.to_bytes();
        let json = String::from_utf8(bytes.to_vec()).map_err(|e| Error {
            inner: Kind::InvalidUtf8(e),
        })?;
        let de = &mut serde_json::Deserializer::from_str(&json);

        let response: InnerResponse = serde_path_to_error::deserialize(de).map_err(|e| Error {
            inner: Kind::Json(e),
        })?;

        if response.errors.is_empty() {
            let response = response.inner.unwrap();
            Ok(
                serde_path_to_error::deserialize(response).map_err(|e| Error {
                    inner: Kind::Json(e),
                })?,
            )
        } else {
            Err(Error {
                inner: Kind::ServerError(response.errors[0].detail.clone()),
            })
        }
    }
}

#[derive(Deserialize)]
struct Detail {
    detail: String,
}

#[derive(Deserialize)]
struct InnerResponse {
    #[serde(default)]
    errors: Vec<Detail>,
    #[serde(flatten)]
    inner: Option<serde_json::Value>,
}

#[derive(thiserror::Error, Debug)]
#[error(transparent)]
pub struct Error {
    #[from]
    inner: Kind,
}

#[derive(thiserror::Error, Debug)]
enum Kind {
    #[error("HTTP Error: {0}")]
    Http(#[source] http::Error),
    #[error("Failed to convert data into response as it's not in UTF-8: {0}")]
    InvalidUtf8(#[source] std::string::FromUtf8Error),
    #[error("Invalid JSON: {0}")]
    Json(#[source] serde_path_to_error::Error<serde_json::Error>),
    #[error("Error while queuing client for request: {0}")]
    Service(#[source] BoxError),
    #[error("{0}")]
    ServerError(String),
    #[error("Unexpected error occurred: {0}")]
    Other(#[source] BoxError),
}
