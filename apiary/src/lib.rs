pub mod closed;
pub mod router;
pub mod server;
pub mod service;

pub type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;
