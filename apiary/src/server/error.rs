use bytes::Bytes;
use http::Request;
use http_body::combinators::BoxBody;

use crate::BoxError;

pub type BoxRequest = Request<BoxBody<Bytes, BoxError>>;

/// 404 Not Found
///
/// It is returned if the request doesn't match
/// with any of the handler methods.
#[derive(Debug, thiserror::Error)]
#[error("404 Not Found")]
pub struct NotFound(pub BoxRequest);

impl From<NotFound> for BoxRequest {
    fn from(v: NotFound) -> Self {
        v.0
    }
}

/// 500 Bad Request - Invalid body
///
/// It is returned if the request's body
/// can't be parsed to the type the handler method expects.
#[derive(Debug, thiserror::Error)]
#[error("500 Bad Request - Invalid body")]
pub struct InvalidBody(pub BoxRequest);

impl From<InvalidBody> for BoxRequest {
    fn from(v: InvalidBody) -> Self {
        v.0
    }
}
