use std::fmt;
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use http::{Request, Response};
use pin_project::pin_project;
use tower::{Service, ServiceBuilder};

pub use tokio::task::JoinHandle;

use crate::closed::{ConnectionCloseGuard, ConnectionClosed};
use crate::server::Server;
use crate::service::ServiceBuilderExt;
use crate::BoxError;

pub(crate) type RespResult = Result<Response<String>, BoxError>;
pub type HandlerResult = JoinHandle<RespResult>;

/// Router type which holds both the Arc-ed app and the routing handler.
pub struct Router<App: ?Sized> {
    pub app: Arc<App>,
    pub handler: fn(Arc<App>, Request<String>, ConnectionClosed) -> HandlerResult,
}

impl<App: Send + Sync + ?Sized + 'static> Router<App> {
    pub async fn run(self, addr: SocketAddr) -> Result<(), BoxError> {
        let service = ServiceBuilder::new().stringify_body().service(self);
        let server = Server::bind(addr, service)?;

        server.run().await
    }
}

#[pin_project]
#[derive(Debug)]
pub struct CallFuture {
    #[pin]
    handle: HandlerResult,
    guard: ConnectionCloseGuard,
}

impl<App: Send + Sync + ?Sized> Service<Request<String>> for Router<App> {
    type Response = Response<String>;
    type Error = crate::BoxError;
    type Future = CallFuture;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<String>) -> Self::Future {
        let (closed, guard) = ConnectionClosed::new();
        let handle = (self.handler)(Arc::clone(&self.app), req, closed);
        CallFuture { handle, guard }
    }
}

impl Future for CallFuture {
    type Output = RespResult;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.project().handle.poll(cx).map(|res| res?)
    }
}

impl<App: ?Sized> Clone for Router<App> {
    fn clone(&self) -> Self {
        Router {
            app: Arc::clone(&self.app),
            handler: self.handler,
        }
    }
}

impl<App: fmt::Debug + ?Sized> fmt::Debug for Router<App> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Router").field("app", &self.app).finish()
    }
}
