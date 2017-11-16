use callback::Callback;
use error::WritiumError;
use request::Request;
use hyper::{Headers, StatusCode};
use serde_json::Value as Json;

pub use super::hyper::Response as HyperResponse;

pub struct Response {
    status: StatusCode,
    headers: Headers,
    body: Option<Json>,
    call_req: Option<Request>,
    callback: Option<Box<Callback>>,
}
impl Response {
    pub fn new(status: StatusCode) -> Response {
        Response {
            headers: Headers::default(),
            status: status,
            body: None,
            call_req: None,
            callback: None,
        }
    }

    pub fn with_header<H: ::hyper::header::Header>(mut self, header: H) -> Self {
        self.headers.set(header);
        self
    }
    pub fn with_headers(mut self, headers: Headers) -> Self {
        self.headers = headers;
        self
    }
    pub fn with_json(mut self, json: Json) -> Self {
        self.body = Some(json);
        self
    }
    pub fn with_call(mut self, req: Request) -> Self {
        self.call_req = Some(req);
        self
    }
    pub fn with_call_back<Cb>(mut self, req: Request, callback: Cb) -> Self
        where Cb: Callback + 'static {
        self.call_req = Some(req);
        self.callback = Some(Box::new(callback));
        self
    }

    pub(crate) fn _take_call_request(&mut self) -> Option<Request> {
        self.call_req.take()
    }
    pub(crate) fn _take_callback_fn(&mut self) -> Option<Box<Callback>> {
        self.callback.take()
    }
}
impl Into<Result<HyperResponse, WritiumError>> for Response {
    fn into(self) -> Result<HyperResponse, WritiumError> {
        use serde_json::to_string;        
        Ok(HyperResponse::new()
            .with_status(self.status)
            .with_headers(self.headers)
            .with_body(to_string(&self.body)?))
    }
}
