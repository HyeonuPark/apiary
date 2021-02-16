use std::future::Future;
use std::mem;
use std::pin::Pin;
use std::string::FromUtf8Error;
use std::task::{Context, Poll};

use futures_core::ready;
use http::request::{self, Request};
use http::{header, Response};
use hyper::body::{Body, HttpBody};
use pin_project::pin_project;

use crate::BoxError;

use super::PolledAfterComplete;

#[derive(Debug, Clone)]
pub struct Service<T> {
    inner: T,
}

#[derive(Debug, Clone)]
pub struct Layer;

#[pin_project(project = Proj)]
#[derive(Debug)]
pub enum CallFuture<T: tower::Service<Request<String>>> {
    Reading {
        head: Option<request::Parts>,
        #[pin]
        body: Body,
        buf: Vec<u8>,
        inner: T,
    },
    Pending(#[pin] T::Future),
    Failed(BoxError),
}

#[derive(Debug, thiserror::Error)]
#[error("Failed to read request body - {source}")]
pub struct ErrorReadRequest {
    #[from]
    source: hyper::Error,
}

#[derive(Debug, thiserror::Error)]
#[error("Request body is not a valid UTF-8 sequence - {source}")]
pub struct ErrorNotUtf8 {
    #[from]
    source: FromUtf8Error,
}

impl<T> tower::Layer<T> for Layer {
    type Service = Service<T>;

    fn layer(&self, inner: T) -> Self::Service {
        Service { inner }
    }
}

impl<T> Service<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }

    pub fn layer() -> Layer {
        Layer
    }

    pub fn get_ref(&self) -> &T {
        &self.inner
    }

    pub fn get_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T> tower::Service<Request<Body>> for Service<T>
where
    T: tower::Service<Request<String>, Response = Response<String>> + Clone,
    T::Error: Into<BoxError>,
{
    type Response = Response<Body>;
    type Error = BoxError;
    type Future = CallFuture<T>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let capacity = length_hint(&req).unwrap_or(0);
        let buf = Vec::with_capacity(capacity);
        let (head, body) = req.into_parts();

        let inner = self.inner.clone();
        // ensure the returned future contains the inner service
        // with the .poll_ready() called actually instead of
        // the fresh new cloned service.
        let inner = mem::replace(&mut self.inner, inner);

        CallFuture::Reading {
            head: Some(head),
            body,
            buf,
            inner,
        }
    }
}

fn length_hint(req: &Request<Body>) -> Option<usize> {
    req.headers()
        .get(&header::CONTENT_LENGTH)?
        .to_str()
        .ok()?
        .parse()
        .ok()
}

impl<T> Future for CallFuture<T>
where
    T: tower::Service<Request<String>, Response = Response<String>> + Clone,
    T::Error: Into<BoxError>,
{
    type Output = Result<Response<Body>, BoxError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            let next = match self.as_mut().project() {
                Proj::Reading {
                    head,
                    body,
                    buf,
                    inner,
                } => match ready!(body.poll_data(cx)) {
                    Some(Err(err)) => Some(CallFuture::Failed(ErrorReadRequest::from(err).into())),
                    Some(Ok(chunk)) => {
                        buf.extend_from_slice(&chunk);
                        None
                    }
                    // TODO: support http/2 trailer headers
                    None => match String::from_utf8(mem::take(buf)) {
                        Err(err) => Some(CallFuture::Failed(ErrorNotUtf8::from(err).into())),
                        Ok(s) => {
                            let head = head.take().expect("request body is terminated twice");
                            let req = Request::from_parts(head, s);
                            let fut = inner.call(req);
                            Some(CallFuture::Pending(fut))
                        }
                    },
                },
                Proj::Pending(fut) => match ready!(fut.poll(cx)) {
                    Ok(res) => return Poll::Ready(Ok(res.map(Into::into))),
                    Err(err) => Some(CallFuture::Failed(err.into())),
                },
                Proj::Failed(err) => {
                    return Poll::Ready(Err(mem::replace(err, PolledAfterComplete.into())))
                }
            };

            if let Some(next) = next {
                self.set(next)
            }
        }
    }
}
