use std::fmt;

use http::StatusCode;

use crate::BoxError;

pub trait Error: Sized {
    fn invalid_request(errors: Vec<BoxError>) -> Self;
    fn status_code(&self) -> StatusCode;
    fn content_type(&self) -> &str;
    fn into_string(self) -> String;
}

impl Error for BoxError {
    fn invalid_request(errors: Vec<BoxError>) -> Self {
        InvalidRequest { errors }.into()
    }

    fn status_code(&self) -> StatusCode {
        if self.is::<InvalidRequest>() {
            StatusCode::BAD_REQUEST
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }

    fn content_type(&self) -> &str {
        "text/plain; charset=utf-8"
    }

    fn into_string(self) -> String {
        self.to_string()
    }
}

#[derive(Debug, thiserror::Error)]
struct InvalidRequest {
    errors: Vec<BoxError>,
}

impl fmt::Display for InvalidRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Failed to parse request with the following errors:")?;

        for err in &self.errors {
            writeln!(f, "  - {}", err)?;
        }

        Ok(())
    }
}
