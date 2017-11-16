use std::collections::{HashMap};
use request::HyperRequest;
use hyper::StatusCode;
use response::HyperResponse;
use super::{Api, Namespace, Request, WritiumError, WritiumResult};

pub struct Writium {
    extra: HashMap<String, String>,
    ns: Namespace,
}
impl Writium {
    pub fn new() -> Writium {
        Writium {
            extra: HashMap::new(),
            ns: Namespace::new(&[]),
        }
    }

    pub fn _route(&self, req: Request) -> WritiumResult {
        // Retrieve response.
        match self.ns.route(req) {
            // Check if we have to call another place or not.
            Ok(mut res) =>
                // An response, we might have to make another call.
                if let Some(call_req) = res._take_call_request() {
                    // It's only unwrapped if there is no more calles required.
                    if let Some(mut cb) = res._take_callback_fn() {
                        cb.callback(self._route(call_req))
                    // No need to call back.
                    } else {
                        self._route(call_req)
                    }
                // Already an error.
                } else {
                    Ok(res)
                },
            Err(err) => Err(err),
        }
    }
    pub fn route(&self, req: HyperRequest) -> HyperResponse {
        // No need to check namespace name; no post processing. Safe to route
        // directly.
        if let Some(req) = Request::new(req) {
            match self._route(req) {
                Ok(res) => match res.into() {
                    Ok(res) => res,
                    // Error may occur on serializing into JSON.
                    Err(err) => err.into(),
                },
                Err(err) => err.into(),
            }
        } else {
            WritiumError::new(StatusCode::BadRequest, "Invalid URL.").into()
        }
    }

    pub fn extra(&self) -> &HashMap<String, String> {
        &self.extra
    }
    pub fn extra_mut(&mut self) -> &mut HashMap<String, String> {
        &mut self.extra
    }

    pub fn bind<A: Api + 'static>(self, api: A) -> Writium {
        Writium {
            extra: self.extra,
            ns: self.ns.bind(api)
        }
    }
}
