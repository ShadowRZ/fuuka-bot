use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Poll, ready},
};

use bytes::Bytes;
use http::{HeaderMap, HeaderValue, Response, StatusCode, Uri, header, request::Request};
use http_body::{Body, Frame, SizeHint};
use http_body_util::{BodyExt, Full};
use octocrab::service::middleware::cache::{CacheKey, CacheStorage, CacheWriter};
use pin_project_lite::pin_project;
use tower::{Layer, Service};

// Adapted from Octocrab
// Implementation based on the documentation at:
// https://docs.github.com/en/rest/using-the-rest-api/best-practices-for-using-the-rest-api?apiVersion=2022-11-28#use-conditional-requests-if-appropriate

#[derive(Clone)]
/// Layer that handles response caching using given [CacheStorage].
pub struct HttpCacheLayer {
    storage: Option<Arc<dyn CacheStorage>>,
}

impl HttpCacheLayer {
    pub fn new(storage: Option<Arc<dyn CacheStorage>>) -> Self {
        HttpCacheLayer { storage }
    }
}

impl<S> Layer<S> for HttpCacheLayer {
    type Service = HttpCache<S>;

    fn layer(&self, inner: S) -> Self::Service {
        HttpCache {
            inner,
            storage: self.storage.clone(),
        }
    }
}

pub struct HttpCache<S> {
    inner: S,
    storage: Option<Arc<dyn CacheStorage>>,
}

impl<S, ReqBody> Service<Request<ReqBody>> for HttpCache<S>
where
    S: Service<Request<ReqBody>, Response = Response<reqwest::Body>>,
{
    type Error = S::Error;
    type Response = S::Response;
    type Future = HttpCacheFuture<S::Future>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<ReqBody>) -> Self::Future {
        let uri = req.uri().clone();

        if let Some(ref storage) = self.storage {
            // If there is a cache record for this URI, add the corresponding
            // header so that GitHub API might send the unmodified response.
            if let Some(key) = storage.try_hit(&uri) {
                match key {
                    CacheKey::ETag(etag) => {
                        req.headers_mut()
                            .append(header::IF_NONE_MATCH, HeaderValue::from_str(&etag).unwrap());
                    }
                    CacheKey::LastModified(last_modified) => {
                        req.headers_mut().append(
                            header::IF_MODIFIED_SINCE,
                            HeaderValue::from_str(&last_modified).unwrap(),
                        );
                    }
                    _ => {}
                }
            }
        }

        HttpCacheFuture {
            inner: self.inner.call(req),
            storage: self.storage.clone(),
            uri,
        }
    }
}

pin_project! {
    pub struct HttpCacheFuture<F> {
        #[pin]
        inner: F,
        storage: Option<Arc<dyn CacheStorage>>,
        uri: Uri,
    }
}

impl<F, E> Future for HttpCacheFuture<F>
where
    F: Future<Output = Result<Response<reqwest::Body>, E>>,
{
    type Output = Result<Response<reqwest::Body>, E>;

    fn poll(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.project();
        let mut response = ready!(this.inner.poll(cx))?;

        if let &mut Some(ref storage) = this.storage {
            if response.status() == StatusCode::NOT_MODIFIED {
                // If the response is indicated as not modified, reuse the body
                // from the cache.
                let cached = storage.load(this.uri).expect("no body for cache hit");

                for (name, value) in cached.headers.iter() {
                    // These headers are missing in the HTTP 304 Not Modified
                    // response from GitHub API, but are important for further
                    // processing.
                    if [header::CONTENT_TYPE, header::CONTENT_LENGTH, header::LINK].contains(name) {
                        response.headers_mut().append(name, value.clone());
                    }
                }

                // Replace the body.
                *response.body_mut() = reqwest::Body::wrap(
                    Full::new(Bytes::from(cached.body)).map_err(|infallible| match infallible {}),
                );
                *response.status_mut() = StatusCode::OK;
            } else {
                // Try to extract a cache header (either ETag or Last-Modified).
                let cache_key = cache_key_extract_from_headers(response.headers());

                if let Some(key) = cache_key {
                    // If there is a cache header, write the whole response body
                    // to the cache while reading it.
                    let writer = storage.writer(this.uri, key, response.headers().clone());
                    let (parts, mut body) = response.into_parts();
                    body = reqwest::Body::wrap(WriteToCacheBody::new(body, writer));
                    response = Response::from_parts(parts, body);
                }
            }
        }

        Poll::Ready(Ok(response))
    }
}

pin_project! {
    struct WriteToCacheBody<B> {
        #[pin]
        inner: B,
        writer: Box<dyn CacheWriter>,
    }
}

impl<B> WriteToCacheBody<B> {
    fn new(inner: B, writer: Box<dyn CacheWriter>) -> Self {
        Self { inner, writer }
    }
}

impl<B> Body for WriteToCacheBody<B>
where
    B: Body<Data = Bytes, Error = reqwest::Error>,
{
    type Data = Bytes;
    type Error = reqwest::Error;

    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        let this = self.project();
        match this.inner.poll_frame(cx) {
            Poll::Ready(frame) => {
                if let Some(Ok(ref data)) = frame {
                    if let Some(data) = data.data_ref() {
                        this.writer.write_body(data);
                    }
                }

                Poll::Ready(frame)
            }
            Poll::Pending => Poll::Pending,
        }
    }

    fn is_end_stream(&self) -> bool {
        self.inner.is_end_stream()
    }

    fn size_hint(&self) -> SizeHint {
        self.inner.size_hint()
    }
}

fn cache_key_extract_from_headers(headers: &HeaderMap) -> Option<CacheKey> {
    // ETag takes precedence over Last-Modified, because the former is more
    // current and accurate.
    //
    // https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/Last-Modified
    headers
        .get(header::ETAG)
        .and_then(|etag| Some(CacheKey::ETag(etag.to_str().ok()?.to_owned())))
        .or_else(|| {
            headers
                .get(header::LAST_MODIFIED)
                .and_then(|last_modified| {
                    Some(CacheKey::LastModified(
                        last_modified.to_str().ok()?.to_owned(),
                    ))
                })
        })
}
