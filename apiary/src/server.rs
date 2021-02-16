use std::future::Future;
use std::net::SocketAddr;

use http::{Request, Response};
use hyper::body::Body;
use hyper::server::accept::Accept;
use hyper::server::Builder;
use tokio::io::{AsyncRead, AsyncWrite};
use tower::{make::Shared, Service};

pub use hyper::server::conn::{AddrIncoming, Http as HttpConfig};

use crate::BoxError;

pub struct Server<S, A = AddrIncoming> {
    service: S,
    accept: A,
    config: HttpConfig,
}

impl<S> Server<S> {
    pub fn bind(addr: SocketAddr, service: S) -> Result<Self, BoxError> {
        let accept = AddrIncoming::bind(&addr)?;

        Ok(Self::with_acceptor(accept, service))
    }
}

impl<S, A> Server<S, A>
where
    A: Accept,
    A::Conn: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    A::Error: Into<BoxError>,
{
    pub fn with_acceptor(accept: A, service: S) -> Self {
        Self {
            service,
            accept,
            config: HttpConfig::new(),
        }
    }

    pub fn config(&mut self) -> &mut HttpConfig {
        &mut self.config
    }

    pub async fn run(self) -> Result<(), BoxError>
    where
        S: Service<Request<Body>, Response = Response<Body>> + Clone + Send + 'static,
        S::Future: Send + 'static,
        S::Error: Into<BoxError>,
    {
        Builder::new(self.accept, self.config)
            .serve(Shared::new(self.service))
            .await?;

        Ok(())
    }

    pub async fn run_until<F>(self, signal: F) -> Result<(), BoxError>
    where
        F: Future<Output = ()>,
        S: Service<Request<Body>, Response = Response<Body>> + Clone + Send + 'static,
        S::Future: Send + 'static,
        S::Error: Into<BoxError>,
    {
        Builder::new(self.accept, self.config)
            .serve(Shared::new(self.service))
            .with_graceful_shutdown(signal)
            .await?;

        Ok(())
    }
}
