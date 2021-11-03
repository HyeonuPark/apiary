use std::future::Future;
use std::net::SocketAddr;

use hyper::server::accept::Accept;
use hyper::server::conn::{AddrIncoming, Http as HttpConfig};
use hyper::server::Builder;
use tokio::io::{AsyncRead, AsyncWrite};
use tower::make::Shared;

use crate::BoxError;

use super::{Server, Service};

#[derive(Debug)]
pub struct Hyper<S: Server, A = AddrIncoming> {
    service: Service<S>,
    accept: A,
    config: HttpConfig,
}

impl<S: Server> Hyper<S, AddrIncoming> {
    pub fn bind(server: S, addr: SocketAddr) -> hyper::Result<Self> {
        Ok(Self::with_acceptor(server, AddrIncoming::bind(&addr)?))
    }
}

impl<S: Server, A> Hyper<S, A> {
    pub fn with_acceptor(server: S, accept: A) -> Self {
        Hyper {
            service: server.into_service(),
            accept,
            config: HttpConfig::new(),
        }
    }

    pub fn config(&mut self) -> &mut HttpConfig {
        &mut self.config
    }
}

impl<S: Server, A> Hyper<S, A>
where
    A: Accept,
    A::Conn: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    A::Error: Into<BoxError>,
{
    pub async fn run(self) -> hyper::Result<()> {
        Builder::new(self.accept, self.config)
            .serve(Shared::new(self.service))
            .await
    }

    pub async fn run_until<F>(self, signal: F) -> hyper::Result<()>
    where
        F: Future<Output = ()>,
    {
        Builder::new(self.accept, self.config)
            .serve(Shared::new(self.service))
            .with_graceful_shutdown(signal)
            .await
    }
}
