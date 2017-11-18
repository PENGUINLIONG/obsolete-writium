///! Writium error.
use std::fmt::{Debug, Display, Formatter, Result as FormatResult};
use std::error::Error;
use serde_json::Error as JsonError;
use proto::HyperResponse;
use hyper::header::Header;
use hyper::{Headers, StatusCode};

pub struct WritiumError {
    headers: Headers,
    status: StatusCode,
    description: &'static str,
}
impl WritiumError {
    pub fn new(status: StatusCode, description: &'static str) -> WritiumError {
        WritiumError {
            status: status,
            headers: Headers::default(),
            description: description,
        }
    }
    pub fn with_header<H: Header>(mut self, header: H) -> Self {
        self.headers.set(header);
        self
    }
    pub fn with_headers(mut self, headers: Headers) -> Self {
        self.headers = headers;
        self
    }
}
impl Debug for WritiumError {
    fn fmt(&self, f: &mut Formatter) -> FormatResult {
        f.write_str(self.description)
    }
}
impl Display for WritiumError {
    fn fmt(&self, f: &mut Formatter) -> FormatResult {
        f.write_str(self.description)
    }
}
impl Error for WritiumError {
    fn description(&self) -> &str {
        self.description
    }
}
impl Into<HyperResponse> for WritiumError {
    fn into(self) -> HyperResponse {
        HyperResponse::new()
            .with_status(self.status)
            .with_headers(self.headers)
            .with_body(format!(r#"{{"msg":"{}"}}"#, self.description))
    }
}
impl From<JsonError> for WritiumError {
    fn from(_: JsonError) -> WritiumError {
        WritiumError::new(StatusCode::InternalServerError, "unable to serialize data into json")
    }
}
