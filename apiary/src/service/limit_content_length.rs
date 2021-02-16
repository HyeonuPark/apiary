use std::future::Future;
use std::mem;
use std::pin::Pin;
use std::task::{Context, Poll};

use http::{header, Request};
use pin_project::pin_project;

use crate::BoxError;

use super::PolledAfterComplete;

#[derive(Debug, Clone)]
pub struct Layer {
    limit: usize,
}

#[derive(Debug, Clone)]
pub struct Service<T> {
    inner: T,
    limit: usize,
}

#[pin_project(project = Proj)]
#[derive(Debug)]
pub enum CallFuture<T: tower::Service<Request<U>>, U> {
    Pending(#[pin] T::Future),
    Failed(BoxError),
}

#[derive(Debug, thiserror::Error)]
#[error("HTTP Request has too large Content-Length - limit: {limit}, found: {found}")]
pub struct ErrorTooLarge {
    limit: usize,
    found: usize,
}

#[derive(Debug, thiserror::Error)]
#[error("HTTP Request doesn't contains the Content-Type header")]
pub struct ErrorNotFound;

#[derive(Debug, thiserror::Error)]
#[error("HTTP Request contains the Content-Type header with non-integral value")]
pub struct ErrorInvalidValue;

impl Layer {
    pub fn new(limit: usize) -> Self {
        Self { limit }
    }
}

impl<T> tower::Layer<T> for Layer {
    type Service = Service<T>;

    fn layer(&self, inner: T) -> Self::Service {
        Service {
            inner,
            limit: self.limit,
        }
    }
}

impl<T> Service<T> {
    pub fn new(inner: T, limit: usize) -> Self {
        Self { inner, limit }
    }

    pub fn layer(limit: usize) -> Layer {
        Layer::new(limit)
    }

    pub fn limit(&self) -> usize {
        self.limit
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

impl<T, U> tower::Service<Request<U>> for Service<T>
where
    T: tower::Service<Request<U>>,
    T::Error: Into<BoxError>,
{
    type Response = T::Response;
    type Error = BoxError;
    type Future = CallFuture<T, U>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<U>) -> Self::Future {
        let len = match req.headers().get(&header::CONTENT_LENGTH) {
            Some(len) => len,
            None => return CallFuture::Failed(ErrorNotFound.into()),
        };
        let len = match len.to_str() {
            Ok(len) => len,
            Err(_) => return CallFuture::Failed(ErrorInvalidValue.into()),
        };
        let len = match len.parse() {
            Ok(len) => len,
            Err(_) => return CallFuture::Failed(ErrorInvalidValue.into()),
        };
        if len > self.limit {
            return CallFuture::Failed(
                ErrorTooLarge {
                    limit: self.limit,
                    found: len,
                }
                .into(),
            );
        }

        CallFuture::Pending(self.inner.call(req))
    }
}

impl<T, U> Future for CallFuture<T, U>
where
    T: tower::Service<Request<U>>,
    T::Future: Future<Output = Result<T::Response, T::Error>>,
    T::Error: Into<BoxError>,
{
    type Output = Result<T::Response, BoxError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.as_mut().project() {
            Proj::Pending(inner) => inner.poll(cx).map_err(Into::into),
            Proj::Failed(err) => Poll::Ready(Err(mem::replace(err, PolledAfterComplete.into()))),
        }
    }
}
