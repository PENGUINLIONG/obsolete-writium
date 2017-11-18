use std::collections::{HashMap};
use proto::{HyperRequest, HyperResponse};
use super::{Api, ApiResult, Namespace, Request, };
use futures::{self, Future, Stream};

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

    pub fn _route(&self, req: Request) -> ApiResult {
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
    pub fn route(&self, req: HyperRequest)
        -> Box<Future<Item=HyperResponse, Error=::hyper::Error>> {
        let (method, uri, _, headers, body) = req.deconstruct();
        // TODO: Deal with that path segments are not parsed.
        let req = Request::construct(method, uri, headers, Box::new(body.concat2())).unwrap();
        // No need to check namespace name; no post processing. Safe to route
        // directly.
        Box::new(match self._route(req) {
            Ok(res) => match res.try_into_hyper() {
                Ok(res) => futures::future::ok(res),
                // Error may occur on serializing into JSON.
                Err(err) => futures::future::ok(err.into()),
            },
            Err(err) => futures::future::ok(err.into()),
        })
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
