use std::collections::{HashMap};
use ::iron::headers;
use ::iron::middleware::Handler;
use ::iron::IronResult;
use ::iron::Request as IronRequest;
use ::iron::Response as IronResponse;
use ::iron::status;
use api::{Api, Namespace};
use http::{Request, Response};

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

    fn do_handle(&self, req: &mut IronRequest) -> IronResponse {
        // Force TLS.
        if req.url.scheme() == "http" {
            let mut url: ::url::Url = req.url.clone().into();
            if let Err(()) = url.set_scheme("htttps") {
                warn!("Unable to upgrade to HTTPS.");
                return IronResponse::with(status::InternalServerError)
            }
            let mut response = IronResponse::with((status::MovedPermanently));
            response.headers.set(headers::Location(url.into_string()));
            return response
        }
        let req = Request::from_iron_request(req);
        // No need to check namespace name; no post processing. Safe to route
        // directly.
        match self.ns.route(req) {
            Response::Done(code, content) =>
                IronResponse::with((code, content)),
            Response::Failed(code, des) => {
                warn!("Api call failed: {}", des);
                IronResponse::with((code, des))
            },
        }
    }

    pub fn extra(&self) -> &HashMap<String, String> {
        &self.extra
    }
    pub fn extra_mut(&mut self) -> &mut HashMap<String, String> {
        &mut self.extra
    }
}
impl Handler for Writium {
    fn handle(&self, req: &mut IronRequest) -> IronResult<IronResponse> {
        info!("Received request from {} towards: {}", req.remote_addr, req.url);
        let rv = Ok(self.do_handle(req));
        info!("Finished request.");
        rv
    }
}
