use http::StatusCode;

use crate::BoxError;

pub trait Schema {}

pub trait RequestBody: Schema + Sized {
    const CONTENT_TYPE: &'static str;

    fn from_string(s: String) -> Result<Self, BoxError>;
}

pub trait ResponseBody: Schema + Sized {
    const CONTENT_TYPE: &'static str;
    const STATUS_CODE: StatusCode;

    fn into_string(self) -> String;
}

pub trait Body: RequestBody + ResponseBody {}

impl Schema for String {}

impl RequestBody for String {
    const CONTENT_TYPE: &'static str = "text/plain; charset=utf-8";

    fn from_string(s: String) -> Result<Self, BoxError> {
        Ok(s)
    }
}

impl ResponseBody for String {
    const CONTENT_TYPE: &'static str = "text/plain; charset=utf-8";
    const STATUS_CODE: StatusCode = StatusCode::OK;

    fn into_string(self) -> String {
        self
    }
}

impl<'a> Schema for &'a str {}

impl<'a> ResponseBody for &'a str {
    const CONTENT_TYPE: &'static str = "text/plain; charset=utf-8";
    const STATUS_CODE: StatusCode = StatusCode::OK;

    fn into_string(self) -> String {
        self.into()
    }
}
