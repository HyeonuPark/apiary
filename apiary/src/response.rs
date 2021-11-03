use http::{header, Response as HttpResponse, StatusCode};

use crate::BoxError;

mod body;

pub use body::Body;

pub(crate) const CONTENT_TYPE_TEXT: &str = "text/plain; charset=utf-8";

pub trait Response {
    fn into_response(self) -> Result<http::Response<Body>, BoxError>;
}

impl<T: Response, E: Response> Response for Result<T, E> {
    fn into_response(self) -> Result<http::Response<Body>, BoxError> {
        match self {
            Ok(t) => t.into_response(),
            Err(e) => {
                let mut resp = e.into_response()?;
                if resp.status() == StatusCode::OK {
                    *resp.status_mut() = StatusCode::INTERNAL_SERVER_ERROR
                }
                Ok(resp)
            }
        }
    }
}

impl Response for () {
    fn into_response(self) -> Result<http::Response<Body>, BoxError> {
        HttpResponse::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, CONTENT_TYPE_TEXT)
            .header(header::CONTENT_LENGTH, 0)
            .body(Body::empty())
            .map_err(|err| Box::new(err) as _)
    }
}

impl Response for String {
    fn into_response(self) -> Result<http::Response<Body>, BoxError> {
        HttpResponse::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, CONTENT_TYPE_TEXT)
            .header(header::CONTENT_LENGTH, self.len())
            .body(Body::once(self))
            .map_err(|err| Box::new(err) as _)
    }
}

impl Response for &'static str {
    fn into_response(self) -> Result<http::Response<Body>, BoxError> {
        HttpResponse::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, CONTENT_TYPE_TEXT)
            .header(header::CONTENT_LENGTH, self.len())
            .body(Body::once(self))
            .map_err(|err| Box::new(err) as _)
    }
}

impl Response for Vec<u8> {
    fn into_response(self) -> Result<http::Response<Body>, BoxError> {
        HttpResponse::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/octet-stream")
            .header(header::CONTENT_LENGTH, self.len())
            .body(Body::once(self))
            .map_err(|err| Box::new(err) as _)
    }
}

impl Response for BoxError {
    fn into_response(self) -> Result<http::Response<Body>, BoxError> {
        let msg = self.to_string();

        HttpResponse::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header(header::CONTENT_TYPE, CONTENT_TYPE_TEXT)
            .header(header::CONTENT_LENGTH, msg.len())
            .body(Body::once(msg))
            .map_err(|err| Box::new(err) as _)
    }
}
