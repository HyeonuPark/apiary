use crate::router::RespResult;

pub use http;

#[derive(Debug, thiserror::Error)]
#[error("404 Not Found")]
pub struct NotFound;

pub async fn default_404() -> RespResult {
    Err(Box::new(NotFound))
}

#[derive(Debug, thiserror::Error)]
#[error("Parameter parse failed")]
pub struct ParseFailed;

pub async fn parse_failed() -> RespResult {
    Err(Box::new(ParseFailed))
}
