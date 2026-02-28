//! Named futures.
use std::{marker::PhantomData, pin::Pin};

use http::{HeaderValue, uri::PathAndQuery};
use serde::de::DeserializeOwned;

/// `IntoFuture` returned by many endpoints on [`crate::PixivClient`].
pub struct GetRequest<'a, T: DeserializeOwned> {
    pub(crate) client: &'a crate::PixivClient,
    pub(crate) path_and_query: Result<PathAndQuery, crate::Error>,
    pub(crate) lang: Option<String>,
    pub(crate) referrer: Result<HeaderValue, crate::Error>,
    pub(crate) _type: PhantomData<T>,
}

impl<T: DeserializeOwned> GetRequest<'_, T> {
    /// Specify the language for this request.
    pub fn with_lang<S: AsRef<str>>(mut self, lang: S) -> Self {
        self.lang = Some(lang.as_ref().to_string());

        self
    }
}

#[allow(clippy::needless_lifetimes)]
impl<'a, T: DeserializeOwned> IntoFuture for GetRequest<'a, T> {
    type Output = crate::Result<T>;
    type IntoFuture = Pin<Box<dyn Future<Output = crate::Result<T>> + Send + 'a>>;

    fn into_future(self) -> Self::IntoFuture {
        let Self {
            client,
            lang,
            referrer,
            path_and_query,
            ..
        } = self;

        Box::pin(async move {
            let path_and_query = path_and_query?;

            let query = match path_and_query.query() {
                Some(query) => lang
                    .map(|lang| format!("{query}&lang={lang}"))
                    .unwrap_or_default(),
                None => lang.map(|lang| format!("lang={lang}")).unwrap_or_default(),
            };
            let path_and_query = PathAndQuery::from_maybe_shared(format!(
                "{path}?{query}",
                path = path_and_query.path()
            ))
            .map_err(Into::<http::Error>::into)?;
            let resp: crate::WrappedResponse<T> = client
                .get_with_referrer_internal(path_and_query, referrer?)
                .await?;

            match resp {
                crate::WrappedResponse::Error(error) => Err(crate::Error::PixivError(error)),
                crate::WrappedResponse::Ok(value) => Ok(value),
            }
        })
    }
}
