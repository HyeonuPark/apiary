use std::future::Future;
#[cfg(feature = "hyper")]
use std::net::SocketAddr;
use std::pin::Pin;
use std::task::{Context, Poll};

use http::{header, Request, Response, StatusCode};
use http_body::Body as HttpBody;

use crate::response;
use crate::BoxError;

#[cfg(feature = "hyper")]
mod with_hyper;

#[cfg(feature = "hyper")]
pub use with_hyper::Hyper;

pub type ServeResult =
    Pin<Box<dyn Future<Output = Result<Response<response::Body>, BoxError>> + Send + 'static>>;

pub trait Server: Clone + Send + 'static {
    fn serve<B: HttpBody>(self, request: Request<B>) -> ServeResult;

    fn into_service(self) -> Service<Self> {
        Service(self)
    }

    #[cfg(feature = "hyper")]
    fn bind(self, addr: SocketAddr) -> hyper::Result<Hyper<Self>> {
        Hyper::bind(self, addr)
    }

    #[cfg(feature = "hyper")]
    fn with_acceptor<A>(self, accept: A) -> Hyper<Self, A> {
        Hyper::with_acceptor(self, accept)
    }
}

#[derive(Debug, Clone)]
pub struct Service<S: Server>(S);

#[derive(Debug)]
pub struct NotFound;

impl<S: Server, B: HttpBody> tower::Service<Request<B>> for Service<S> {
    type Response = Response<response::Body>;
    type Error = BoxError;
    type Future = ServeResult;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        self.0.clone().serve(req)
    }
}

impl crate::response::Response for NotFound {
    fn into_response(self) -> Result<Response<response::Body>, BoxError> {
        let resp = "404 Not Found";

        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .header(header::CONTENT_TYPE, crate::response::CONTENT_TYPE_TEXT)
            .header(header::CONTENT_LENGTH, resp.len())
            .body(response::Body::once(resp))
            .map_err(|err| Box::new(err) as _)
    }
}
