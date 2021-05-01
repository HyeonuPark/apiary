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

pub use http::{header, Method, StatusCode};
pub use tower::ServiceBuilder;

#[cfg(feature = "macro")]
pub use apiary_macro::apiary;

// only for the internal use
#[cfg(feature = "macro")]
#[doc(hidden)]
pub mod internal_helper;
