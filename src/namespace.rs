use hyper::StatusCode;
use super::{Api, ApiResult, Request, WritiumError};

/// `Namespace` bind with apis and forms an intermediate layer of API. It self
/// doesn't do a thing but it will.
///
/// It's name is used to distinguish with other peer APIs and all the sub-API
/// will not see the namespace name segments in the request. If you want the
/// namespace it self to have some functionalities, you need to create a sub-API
/// and name it `&[]`. But such design is not recommended because it sometimes
/// will make the API work in a weird way, especially when path variables are
/// involved, i.e. the trailing part of the path is used as a variable.
pub struct Namespace {
    name: &'static [&'static str],
    apis: Vec<Box<Api>>,
}
impl Namespace {
    pub fn new(name: &'static [&'static str]) -> Namespace {
        Namespace {
            name: name,
            apis: Vec::new(),
        }
    }

    pub fn bind<A: Api>(mut self, api: A) -> Namespace {
        self.apis.push(Box::new(api) as Box<Api>);
        self
    }
}
impl Api for Namespace {
    fn name(&self) -> &'static [&'static str] {
        self.name
    }
    /// The route function here will ask every sub-API to make an response in
    /// binding order. The collection routing is short-circuiting, i.e., once a
    /// sub-API responded, the response is returned and the following it won't
    /// check the remaining unchecked sub-apis.
    fn route(&self, mut req: Request) -> ApiResult {
        use api::RouteHint;
        for api in self.apis.iter() {
            match api.preroute(req) {
                RouteHint::CallMe(r) =>
                    return api.postroute(api.route(r)),
                RouteHint::NotMe(r) => req = r,
            }
        }
        WritiumError::new(StatusCode::NotFound, "api not found").into()
    }
}
