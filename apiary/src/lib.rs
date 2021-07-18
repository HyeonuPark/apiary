pub mod router;
pub mod server;
pub mod service;

pub mod body;
pub mod closed;
pub mod error;

pub type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

pub use body::Body;
pub use closed::ConnectionClosed;
pub use error::Error;
pub use router::Router;
pub use server::Server;
pub use service::ServiceBuilderExt;

pub use http::{header, Method, Request, Response, StatusCode};
pub use tower::ServiceBuilder;

#[cfg(feature = "macro")]
pub use apiary_macro::api;

// only for the internal use
#[cfg(feature = "macro")]
#[doc(hidden)]
pub mod internal_helper;

pub async fn default_404_not_found(req: Request<String>) -> Result<Response<String>, BoxError> {
    Ok(Response::new(req.into_body()))
}
