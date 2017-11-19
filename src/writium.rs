use std::sync::Arc;
use proto::{HyperRequest, HyperResponse};
use super::{Api, ApiResult, FutureExt, Namespace, Request, ok};
use futures::Future;

pub struct Writium {
    ns: Arc<Namespace>,
}
impl Writium {
    pub fn new() -> Writium {
        Writium {
            ns: Arc::new(Namespace::new(&[])),
        }
    }

    pub fn route(&self, req: HyperRequest)
        -> Box<Future<Item=HyperResponse, Error=::hyper::Error>> {
        let (method, uri, _, headers, body) = req.deconstruct();
        // TODO: Deal with that path segments are not parsed.
        let req = Request::construct(method, uri, headers, body).unwrap();
        // No need to check namespace name; no post processing. Safe to route
        // directly.
        _route(self.ns.clone(), req).then(|result|
            -> Result<HyperResponse, ::hyper::Error> {
            match result {
                Ok(res) => match res.try_into_hyper() {
                    Ok(res) => Ok(res),
                    // Error may occur on serializing into JSON.
                    Err(err) => Ok(err.into()),
                },
                Err(e) => Ok(e.into()),
            }
        }).into_box()
    }

    pub fn bind<A: Api + 'static>(&mut self, api: A) {
        Arc::make_mut(&mut self.ns).bind(api)
    }
}
fn _route(ns: Arc<Namespace>, req: Request) -> ApiResult {
    // Retrieve response.
    ns.route(req).and_then(|mut res| {
            // Check if we have to call another place or not.
            // An response, we might have to make another call.
            if let Some(call_req) = res._take_call_request() {
                // It's only unwrapped if there is no more calles required.
                if let Some(mut cb) = res._take_callback_fn() {
                    cb.callback(_route(ns, call_req))
                // No need to call back.
                } else {
                    _route(ns, call_req)
                }
            // Already an error.
            } else {
                ok(res)
            }
        })
        .into_box()
}
