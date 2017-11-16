#![allow(unused_variables)]
use hyper::{Method, StatusCode};
use super::{Request, WritiumResult, WritiumError};

const NOT_SUPPORTED: &str = "not supported";

pub enum RouteHint {
    CallMe(Request),
    NotMe(Request),
}

pub type ApiName = &'static[&'static str];

fn gen_not_implemented() -> WritiumResult {
    WritiumError::new(StatusCode::MethodNotAllowed, NOT_SUPPORTED).into()
}
pub trait Api: 'static + Send + Sync {
    /// Name of API. It identifies an API and allow Writium to route by URL path
    /// segments.
    fn name(&self) -> ApiName;

    /// Do things before function routing, like processing headers or content.
    /// By default, it checks if the namespace in url is about the current API.
    /// None should be returned if the call is unrelated to the current API.
    //
    /// Note: `writium.do_handle(~)` must be updated if its default behavior is
    /// changed.
    fn preroute(&self, mut req: Request) -> RouteHint {
        // Reject if the path is not prefixed by name of the current API.
        for seg in self.name() {
            if req.route_seg(Some(seg)).is_none() {
                return RouteHint::NotMe(req)
            }
        }
        RouteHint::CallMe(req)
    }

    /// Route request to corresponding method. It's recommended not to override
    /// the default implementation. `None` is returned when the requested api is
    /// not the current one.
    ///
    /// Note: `writium.do_handle(~)` must be updated if its default behavior is
    /// changed.
    fn route(&self, req: Request) -> WritiumResult {
        // Remove path prefix.
        info!("API found: /{}", self.name().join("/"));
        // Dispatch APIs.
        match req.as_ref().method().clone() {
            Method::Get =>    self.get   (req),
            Method::Delete => self.delete(req),
            Method::Patch =>  self.patch (req),
            Method::Post =>   self.post  (req),
            Method::Put =>    self.put   (req),
            _ => WritiumError::new(StatusCode::MethodNotAllowed, NOT_SUPPORTED).into()
        }
    }

    /// Do things after funciton routing, like replaceing a failure with a
    /// default response. By default, it does nothing.
    fn postroute(&self, res: WritiumResult) -> WritiumResult {
        res
    }

    /// Process DELETE method.
    fn delete(&self, req: Request) -> WritiumResult {
        gen_not_implemented()
    }
    /// Process GET    method.
    fn get   (&self, req: Request) -> WritiumResult {
        gen_not_implemented()
    }
    /// Process PATCH  method.
    fn patch (&self, req: Request) -> WritiumResult {
        gen_not_implemented()
    }
    /// Process POST   method.
    fn post  (&self, req: Request) -> WritiumResult {
        gen_not_implemented()
    }
    /// Process PUT    method.
    fn put   (&self, req: Request) -> WritiumResult {
        gen_not_implemented()
    }
}
