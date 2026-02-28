pub mod common;
pub mod futures;
pub mod illust;
pub mod ranking;
mod serde;

use std::{marker::PhantomData, str::FromStr};

use ::serde::{Deserialize, Deserializer, Serialize, de::DeserializeOwned};
use bytes::Bytes;
use futures_core::{Stream, future::BoxFuture};
use http::{HeaderValue, Request, Response, Uri, uri::PathAndQuery};
use http_body_util::BodyExt;
use secrecy::{ExposeSecret, SecretString};
use tower::{BoxError, Service, ServiceExt, buffer::Buffer, util::BoxService};

use crate::{
    futures::GetRequest,
    illust::IllustInfo,
    ranking::{Ranking, RankingContent, RankingItem, RankingMode},
};

/// Holds a date time raw string.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[serde(transparent)]
pub struct DateTime(pub String);

/// A `Result` alias where the `Err` case is `pixrs::Error`.
pub type Result<T> = std::result::Result<T, crate::Error>;

pub type BoxBody = http_body_util::combinators::BoxBody<Bytes, crate::Error>;

type PixivService = Buffer<Request<BoxBody>, BoxFuture<'static, Result<Response<BoxBody>>>>;

static BASE_URL: &str = "https://www.pixiv.net";
static USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36";

#[derive(Clone)]
pub struct PixivClient {
    token: SecretString,
    service: PixivService,
}

impl PixivClient {
    pub fn new<S>(service: S, token: SecretString) -> Self
    where
        S: tower::Service<http::Request<BoxBody>, Response = http::Response<BoxBody>>
            + Send
            + 'static,
        S::Future: Send + 'static,
        S::Error: Into<crate::Error>,
    {
        let service = Buffer::new(BoxService::new(service.map_err(Into::into)), 1024);

        Self { service, token }
    }

    /// Send a `GET` request with no additional post-processing.
    pub async fn _get(&self, uri: impl TryInto<Uri>) -> Result<Response<BoxBody>> {
        self._get_with_referrer(uri, "https://www.pixiv.net/").await
    }

    /// Send a `GET` request with no additional post-processing, passing a referrer.
    async fn _get_with_referrer<R>(
        &self,
        uri: impl TryInto<Uri>,
        referrer: R,
    ) -> Result<Response<BoxBody>>
    where
        R: TryInto<HeaderValue>,
        <R as TryInto<HeaderValue>>::Error: Into<http::Error>,
    {
        use http::header::{COOKIE, REFERER, USER_AGENT};

        let uri = uri.try_into().map_err(|_| Error::UriParse)?;
        let mut cookie = HeaderValue::from_maybe_shared(format!(
            "PHPSESSID={token}",
            token = self.token.expose_secret()
        ))
        .map_err(Into::<http::Error>::into)?;
        cookie.set_sensitive(true);

        let request = http::Request::builder()
            .uri(uri)
            .header(REFERER, referrer)
            .header(USER_AGENT, crate::USER_AGENT)
            .header(COOKIE, cookie)
            .body(
                http_body_util::Empty::<Bytes>::new()
                    .map_err(|e| Error::Other(Box::new(e) as BoxError))
                    .boxed(),
            )?;

        let mut service = self.service.clone();
        service
            .ready()
            .await
            .map_err(Error::Service)?
            .call(request)
            .await
            .map_err(Error::Service)
    }

    /// Send a `GET` request to `route` with optional query parameters, returning raw bytes.
    pub async fn get_raw<A, P>(&self, route: A, parameters: Option<&P>) -> Result<Bytes>
    where
        A: AsRef<str>,
        P: Serialize + ?Sized,
    {
        self.get_raw_with_referrer(route, parameters, "https://www.pixiv.net")
            .await
    }

    /// Send a `GET` request to `route` with optional query parameters and referrer, returning raw bytes.
    async fn get_raw_with_referrer<A, P, R>(
        &self,
        route: A,
        parameters: Option<&P>,
        referrer: R,
    ) -> Result<Bytes>
    where
        A: AsRef<str>,
        P: Serialize + ?Sized,
        R: TryInto<HeaderValue>,
        <R as TryInto<HeaderValue>>::Error: Into<http::Error>,
    {
        self.get_raw_with_referrer_internal(self.parameterized_uri(route, parameters)?, referrer)
            .await
    }

    /// Send a `GET` request to `route` with optional query parameters, returning the body of the response.
    pub async fn get<A, P, R>(&self, route: A, parameters: Option<&P>) -> Result<R>
    where
        A: AsRef<str>,
        P: Serialize + ?Sized,
        R: DeserializeOwned,
    {
        self.get_with_referrer(route, parameters, "https://www.pixiv.net/")
            .await
    }

    /// Send a `GET` request to `route` with optional query parameters, returning the body of the response.
    async fn get_with_referrer<A, P, R, Referrer>(
        &self,
        route: A,
        parameters: Option<&P>,
        referrer: Referrer,
    ) -> Result<R>
    where
        A: AsRef<str>,
        P: Serialize + ?Sized,
        R: DeserializeOwned,
        Referrer: TryInto<HeaderValue>,
        <Referrer as TryInto<HeaderValue>>::Error: Into<http::Error>,
    {
        self.get_with_referrer_internal(self.parameterized_uri(route, parameters)?, referrer)
            .await
    }

    pub(crate) async fn get_with_referrer_internal<R, Referrer>(
        &self,
        path_and_query: PathAndQuery,
        referrer: Referrer,
    ) -> Result<R>
    where
        R: DeserializeOwned,
        Referrer: TryInto<HeaderValue>,
        <Referrer as TryInto<HeaderValue>>::Error: Into<http::Error>,
    {
        let bytes = self
            .get_raw_with_referrer_internal(path_and_query, referrer)
            .await?;
        let json = String::from_utf8(bytes.to_vec()).map_err(Error::InvalidUtf8)?;
        let de = &mut serde_json::Deserializer::from_str(&json);
        let response = serde_path_to_error::deserialize(de).map_err(Error::Json)?;

        Ok(response)
    }

    pub(crate) async fn get_raw_with_referrer_internal<R>(
        &self,
        path_and_query: PathAndQuery,
        referrer: R,
    ) -> Result<Bytes>
    where
        R: TryInto<HeaderValue>,
        <R as TryInto<HeaderValue>>::Error: Into<http::Error>,
    {
        let mut parts = Uri::from_static(BASE_URL).into_parts();
        parts.path_and_query = Some(path_and_query);

        let response = self._get_with_referrer(parts, referrer).await?;

        let body = response.into_body();
        let bytes = body.collect().await?.to_bytes();

        Ok(bytes)
    }

    async fn _ranking(
        &self,
        mode: RankingMode,
        content: RankingContent,
        date: &Option<String>,
        page: Option<u32>,
    ) -> Result<Ranking> {
        let mode = match mode {
            RankingMode::Daily => "&mode=daily",
            RankingMode::Weekly => "&mode=weekly",
            RankingMode::Monthly => "&mode=monthly",
            RankingMode::Rookie => "&mode=rookie",
            RankingMode::Original => "&mode=original",
            RankingMode::Male => "&mode=male",
            RankingMode::Female => "&mode=female",
            RankingMode::DailyR18 => "&mode=daily_r18",
            RankingMode::WeeklyR18 => "&mode=weekly_r18",
            RankingMode::MaleR18 => "&mode=male_r18",
            RankingMode::FemaleR18 => "&mode=female_r18",
            RankingMode::R18G => "&mode=r18g",
        };
        let content = match content {
            RankingContent::All => "",
            RankingContent::Illust => "&content=illust",
            RankingContent::Ugoira => "&content=ugoira",
            RankingContent::Manga => "&content=manga",
        };
        let page = page.map(|p| format!("&p={p}")).unwrap_or_default();
        let date = date
            .as_ref()
            .map(|d| format!("&date={d}"))
            .unwrap_or_default();

        self.get_with_referrer_internal(
            PathAndQuery::from_maybe_shared(format!(
                "/ranking.php?format=json{mode}{content}{page}{date}"
            ))
            .map_err(Into::<http::Error>::into)?,
            "https://www.pixiv.net/ranking.php",
        )
        .await
    }

    // Public API.

    /// Get the info of an illust.
    pub fn illust_info(&self, illust_id: i32) -> GetRequest<'_, IllustInfo> {
        GetRequest {
            client: self,
            path_and_query: self
                .parameterized_uri(format!("/ajax/illust/{illust_id}"), None as Option<&()>),
            lang: None,
            referrer: HeaderValue::from_maybe_shared(format!(
                "https://www.pixiv.net/artworks/{illust_id}"
            ))
            .map_err(Into::<http::Error>::into)
            .map_err(Into::into),
            _type: PhantomData,
        }
    }

    /// Get the User ID of the logged in user.
    pub async fn self_user_id(&self) -> Result<Option<u64>> {
        let resp = self._get(BASE_URL).await?;
        let headers = resp.headers();
        Ok(headers
            .get("x-userid")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| <u64 as FromStr>::from_str(value).ok()))
    }

    /// Get the Pixiv ranking.
    pub async fn ranking(
        &self,
        mode: RankingMode,
        content: RankingContent,
        date: Option<String>,
        page: Option<u32>,
    ) -> Result<Ranking> {
        self._ranking(mode, content, &date, page).await
    }

    /// Get the Pixiv ranking as a series of stream.
    pub async fn ranking_stream(
        &self,
        mode: RankingMode,
        content: RankingContent,
        date: Option<String>,
    ) -> impl Stream<Item = Result<RankingItem>> + '_ {
        async_stream::try_stream! {
            let first = self._ranking(mode, content, &date, None).await?;
            for content in first.contents {
                yield content;
            }
            while let Some(next) = first.next {
                let result = self._ranking(mode, content, &date, Some(next)).await?;
                for content in result.contents {
                    yield content;
                }
            }
        }
    }

    /// Convenience method to accept any &str, and attempt to convert it to a PathAndQuery.
    /// the method also attempts to serialize any parameters into a query string, and append it to the uri.
    fn parameterized_uri<A, P>(&self, uri: A, parameters: Option<&P>) -> Result<PathAndQuery>
    where
        A: AsRef<str>,
        P: Serialize + ?Sized,
    {
        let mut uri = uri.as_ref().to_string();
        if let Some(parameters) = parameters {
            if uri.contains('?') {
                uri = format!("{uri}&");
            } else {
                uri = format!("{uri}?");
            }
            uri = format!(
                "{}{}",
                uri,
                serde_urlencoded::to_string(parameters)?.as_str()
            );
        }
        let uri = PathAndQuery::from_str(uri.as_str()).map_err(Into::<http::Error>::into)?;

        Ok(uri)
    }
}

pub(crate) enum WrappedResponse<T: DeserializeOwned> {
    Error(String),
    Ok(T),
}

impl<'de, T: DeserializeOwned> Deserialize<'de> for WrappedResponse<T> {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[allow(unused)]
        #[derive(Deserialize)]
        #[serde(untagged, bound = "T: DeserializeOwned")]
        enum WrappedResponse<T: DeserializeOwned> {
            Error {
                error: serde_bool::True,
                message: String,
            },
            Ok {
                error: serde_bool::False,
                body: T,
            },
        }

        WrappedResponse::deserialize(deserializer).map(|result: WrappedResponse<T>| match result {
            WrappedResponse::Error { message, .. } => Self::Error(message),
            WrappedResponse::Ok { body, .. } => Self::Ok(body),
        })
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to parse URI")]
    UriParse,
    #[error("HTTP Error")]
    Http(
        #[source]
        #[from]
        http::Error,
    ),
    #[error("Failed to convert data into response as it's not in UTF-8")]
    InvalidUtf8(
        #[source]
        #[from]
        std::string::FromUtf8Error,
    ),
    #[error("Invalid JSON")]
    Json(
        #[source]
        #[from]
        serde_path_to_error::Error<serde_json::Error>,
    ),
    #[error("Building query failed")]
    BuildQuery(
        #[source]
        #[from]
        serde_urlencoded::ser::Error,
    ),
    #[error("Error while queuing client for request")]
    Service(#[source] BoxError),
    #[error("{0}")]
    PixivError(String),
    #[error(transparent)]
    Other(#[from] BoxError),
}
