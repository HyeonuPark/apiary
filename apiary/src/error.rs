use http::StatusCode;

use crate::BoxError;

pub trait Error {
    fn status_code(&self) -> StatusCode;
    fn content_type(&self) -> &str;
    fn into_string(self) -> String;
}

impl Error for String {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }

    fn content_type(&self) -> &str {
        "text/plain; charset=utf-8"
    }

    fn into_string(self) -> String {
        self
    }
}

impl Error for BoxError {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }

    fn content_type(&self) -> &str {
        "text/plain; charset=utf-8"
    }

    fn into_string(self) -> String {
        self.to_string()
    }
}
