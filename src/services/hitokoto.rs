//! Hitokoto API.
//!
//! > 动漫也好、小说也好、网络也好，不论在哪里，我们总会看到有那么一两个句子能穿透你的心。我们把这些句子汇聚起来，形成一言网络，以传递更多的感动。如果可以，我们希望我们没有停止服务的那一天。
//! >
//! > 简单来说，一言指的就是一句话，可以是动漫中的台词，也可以是网络上的各种小段子。 或是感动，或是开心，有或是单纯的回忆。来到这里，留下你所喜欢的那一句句话，与大家分享，这就是一言存在的目的。
use std::collections::BTreeSet;
use std::str::FromStr;

use bytes::Bytes;
use http_body_util::BodyExt;
use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;
use serde::{Deserialize, Serialize};
use tower::{BoxError, buffer::Buffer, util::BoxService};
use tower::{Service, ServiceExt};
use tower_http::ServiceBuilderExt;
use url::Url;

type BoxBody = http_body_util::combinators::BoxBody<Bytes, Error>;

pub type HitokotoService = Buffer<
    http::Request<BoxBody>,
    <BoxService<http::Request<BoxBody>, http::Response<BoxBody>, BoxError> as tower::Service<
        http::Request<BoxBody>,
    >>::Future,
>;

/// <https://developer.hitokoto.cn/sentence/#%E8%BF%94%E5%9B%9E%E4%BF%A1%E6%81%AF>
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct Response {
    pub id: u64,
    pub uuid: String,
    pub hitokoto: String,
    #[serde(rename = "type")]
    pub type_: Type,
    pub from: String,
    pub from_who: Option<String>,
    pub creator: String,
    pub creator_uid: u64,
    pub reviewer: u64,
    pub commit_from: String,
    pub created_at: String,
    pub length: u64,
}

/// <https://developer.hitokoto.cn/sentence/#%E5%8F%A5%E5%AD%90%E7%B1%BB%E5%9E%8B-%E5%8F%82%E6%95%B0>
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Type {
    /// 动画
    #[serde(rename = "a")]
    Anime,
    /// 漫画
    #[serde(rename = "b")]
    Comic,
    /// 游戏
    #[serde(rename = "c")]
    Game,
    /// 文学
    #[serde(rename = "d")]
    Literature,
    /// 原创
    #[serde(rename = "e")]
    Original,
    /// 来自网络
    #[serde(rename = "f")]
    Internet,
    /// 其他
    #[serde(rename = "g")]
    Other,
    /// 影视
    #[serde(rename = "h")]
    Video,
    /// 诗词
    #[serde(rename = "i")]
    Poetry,
    /// 网易云
    #[serde(rename = "j")]
    NetEase,
    /// 哲学
    #[serde(rename = "k")]
    Philosophy,
    /// 抖机灵
    #[serde(rename = "l")]
    Joke,
}

pub fn format(resp: Response) -> RoomMessageEventContent {
    let from_who = resp.from_who.unwrap_or_default();

    RoomMessageEventContent::text_html(
        format!(
            "『{0}』——{1}「{2}」\nFrom https://hitokoto.cn/?uuid={3}",
            resp.hitokoto, from_who, resp.from, resp.uuid
        ),
        format!(
            "<p><b>『{0}』</b><br/>——{1}「{2}」</p><p>From https://hitokoto.cn/?uuid={3}</p>",
            resp.hitokoto, from_who, resp.from, resp.uuid
        ),
    )
}

pub async fn request(client: &reqwest::Client, base: Url) -> anyhow::Result<Response> {
    tracing::Span::current().record(
        "fuuka_bot.hitokoto.base_url",
        tracing::field::display(&base),
    );
    let raw = client.get(base).send().await?.error_for_status()?;
    let resp: Response = raw.json().await?;

    Ok(resp)
}

#[derive(Clone)]
pub struct HitokotoClient {
    base_url: http::Uri,
    service: HitokotoService,
}

impl HitokotoClient {
    pub fn from_reqwest_client(client: &reqwest::Client, base_url: http::Uri) -> Self {
        let service = tower::ServiceBuilder::new()
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

    pub async fn request(&self, types: BTreeSet<Type>) -> Result<Response, Error> {
        use http::uri::PathAndQuery;

        const LITERAL_BASE: PathAndQuery = PathAndQuery::from_static("/");
        let path_and_query = self
            .base_url
            .path_and_query()
            .cloned()
            .unwrap_or(LITERAL_BASE);

        let queries: Vec<_> = types
            .into_iter()
            .map(|ty| {
                let str = match ty {
                    Type::Anime => "a",
                    Type::Comic => "b",
                    Type::Game => "c",
                    Type::Literature => "d",
                    Type::Original => "e",
                    Type::Internet => "f",
                    Type::Other => "g",
                    Type::Video => "h",
                    Type::Poetry => "i",
                    Type::NetEase => "j",
                    Type::Philosophy => "k",
                    Type::Joke => "l",
                };

                format!("c={str}")
            })
            .collect();

        let query = queries.join("&");
        let query = match path_and_query.query() {
            Some(base_query) => format!("?{base_query}&{query}"),
            None => "".to_string(),
        };

        let path_and_query =
            PathAndQuery::from_str(&format!("{path}{query}", path = path_and_query.path()))
                .map_err(|e| Error {
                    inner: Kind::Uri(e),
                })?;

        let mut parts = self.base_url.clone().into_parts();
        parts.path_and_query.replace(path_and_query);
        let uri = http::Uri::from_parts(parts).map_err(|e| Error {
            inner: Kind::UriParts(e),
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
        let response = serde_path_to_error::deserialize(de).map_err(|e| Error {
            inner: Kind::Json(e),
        })?;

        Ok(response)
    }
}

#[derive(thiserror::Error, Debug)]
#[error(transparent)]
pub struct Error {
    #[from]
    inner: Kind,
}

#[derive(thiserror::Error, Debug)]
enum Kind {
    #[error("Invalid URI")]
    Uri(#[source] http::uri::InvalidUri),
    #[error("Invalid URI")]
    UriParts(#[source] http::uri::InvalidUriParts),
    #[error("HTTP Error")]
    Http(#[source] http::Error),
    #[error("Failed to convert data into response as it's not in UTF-8")]
    InvalidUtf8(#[source] std::string::FromUtf8Error),
    #[error("Invalid JSON")]
    Json(#[source] serde_path_to_error::Error<serde_json::Error>),
    #[error("Error while queuing client for request")]
    Service(#[source] BoxError),
    #[error(transparent)]
    Other(BoxError),
}
