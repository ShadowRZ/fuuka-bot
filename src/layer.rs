//! [tower] Layers.

use std::{
    pin::Pin,
    task::{Context, Poll},
};

use bytes::Bytes;
use pin_project_lite::pin_project;
use tower::{Layer, Service};

/// Wrap a [reqwest::Client], providing a generic HTTP client.
pub struct ReqwestLayer;

impl Layer<reqwest::Client> for ReqwestLayer {
    type Service = ReqwestService;

    fn layer(&self, inner: reqwest::Client) -> Self::Service {
        ReqwestService { inner }
    }
}

/// Wraps a [reqwest::Client], providing a generic HTTP client as a [tower::Service].
pub struct ReqwestService {
    inner: reqwest::Client,
}

impl<B> Service<http::Request<B>> for ReqwestService
where
    B: http_body::Body + Send + Sync + 'static,
    B::Data: Into<Bytes>,
    B::Error: Into<tower::BoxError>,
{
    type Response = http::Response<reqwest::Body>;

    type Error = reqwest::Error;

    type Future = ReqwestServiceFuture<reqwest::Client>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: http::Request<B>) -> Self::Future {
        let fut = reqwest::Request::try_from(req.map(reqwest::Body::wrap))
            .map(|reqw| self.inner.call(reqw));
        ReqwestServiceFuture::new(fut)
    }
}

pin_project! {
    #[project = ReqwestServiceFutureProj]
    pub enum ReqwestServiceFuture<S>
    where
        S: Service<reqwest::Request>,
    {
        Future {
            #[pin]
            fut: S::Future,
        },
        Error {
            error: Option<reqwest::Error>,
        },
    }
}

impl<S> ReqwestServiceFuture<S>
where
    S: Service<reqwest::Request>,
{
    fn new(future: Result<S::Future, reqwest::Error>) -> Self {
        match future {
            Ok(fut) => Self::Future { fut },
            Err(error) => Self::Error { error: Some(error) },
        }
    }
}

impl<S> Future for ReqwestServiceFuture<S>
where
    S: Service<reqwest::Request, Error = reqwest::Error>,
    http::Response<reqwest::Body>: From<S::Response>,
    reqwest::Error: From<S::Error>,
{
    type Output = Result<http::Response<reqwest::Body>, reqwest::Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        match self.project() {
            ReqwestServiceFutureProj::Future { fut } => fut.poll(cx).map_ok(From::from),
            ReqwestServiceFutureProj::Error { error } => {
                let error = error.take().expect("Polled after ready");
                Poll::Ready(Err(error))
            }
        }
    }
}
