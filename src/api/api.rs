#![allow(unused_variables)]
use http::{Request, Response};
use http::status;

const NOT_SUPPORTED: &str = "not supported";

#[derive(Debug)]
pub enum RouteHint {
    CallMe(Request),
    NotMe(Request),
}

pub type ApiName = &'static[&'static str];
pub type ApiDependencies = &'static[&'static[&'static str]];

fn gen_not_implemented() -> Response {
    Response::Failed(status::MethodNotAllowed, NOT_SUPPORTED)
}
pub trait Api: 'static + Send + Sync {
    /// Name of API. It identifies an API and allow Writium to route by URL path
    /// segments.
    fn name(&self) -> ApiName;
    /// A list of API dependencies. These dependencies will be checked before
    /// writium become available for iron to run with. By default there is no
    /// dependencies.
    fn dependencies(&self) -> ApiDependencies {
        &[]
    }

    /// Do things before function routing, like processing headers or content.
    /// By default, it checks if the namespace in url is about the current API.
    /// None should be returned if the call is unrelated to the current API.
    //
    /// Note: `writium.do_handle(~)` must be updated if its default behavior is
    /// changed.
    fn preroute(&self, req: Request) -> RouteHint {
        fn is_about_me(req: &Request, name: &'static[&'static str]) -> bool {
            let mut req_it = req.path().iter();
            for name_seg in name {
                if let Some(req_seg) = req_it.next() {
                    if name_seg != req_seg {
                        return false
                    }
                } else {
                    return false
                }
            }
            true
        }
        // Reject if the path is not prefixed by name of the current API.
        if is_about_me(&req, self.name()) {
            RouteHint::CallMe(req)
        } else {
            RouteHint::NotMe(req)
        }
    }

    /// Route request to corresponding method. It's recommended not to override
    /// the default implementation. `None` is returned when the requested api is
    /// not the current one.
    ///
    /// Note: `writium.do_handle(~)` must be updated if its default behavior is
    /// changed.
    fn route(&self, mut req: Request) -> Response {
        // Remove path prefix.
        info!("API found: /{}", self.name().join("/"));
        for _ in 0..self.name().len() { req.path_mut().remove(0); }
        use http::method::Method;
        // Dispatch APIs.
        match req.method().clone() {
            Method::Get =>    self.get   (req),
            Method::Delete => self.delete(req),
            Method::Patch =>  self.patch (req),
            Method::Post =>   self.post  (req),
            Method::Put =>    self.put   (req),
            _ => Response::Failed(status::MethodNotAllowed, NOT_SUPPORTED),
        }
    }

    /// Do things after funciton routing, like replaceing failures with a
    /// default response. By default, it does nothing.
    fn postroute(&self, res: Response) -> Response {
        res
    }

    /// Process DELETE method.
    fn delete(&self, req: Request) -> Response { gen_not_implemented() }
    /// Process GET    method.
    fn get   (&self, req: Request) -> Response { gen_not_implemented() }
    /// Process PATCH  method.
    fn patch (&self, req: Request) -> Response { gen_not_implemented() }
    /// Process POST   method.
    fn post  (&self, req: Request) -> Response { gen_not_implemented() }
    /// Process PUT    method.
    fn put   (&self, req: Request) -> Response { gen_not_implemented() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::http::method::Method;    
    struct TestApi;
    impl TestApi { fn new() -> TestApi { TestApi {} } }
    fn res(answer: &str) -> Response {
        Response::Done(status::Ok, answer.to_string())
    }
    fn req(method: Method) -> Request {
        let mut rv = Request::new(method);
        rv.path_mut().push("foo".to_string());
        rv.path_mut().push("bar".to_string());
        rv
    }
    fn bad_req() -> Request {
        let mut rv = Request::new(Method::Get);
        rv.path_mut().push("foo".to_string());
        rv.path_mut().push("not_so_bar".to_string());
        rv
    }
    impl Api for TestApi {
        fn name(&self) -> ApiName {
            &["foo", "bar"]
        }
        fn delete(&self, req: Request) -> Response { res("delete") }
        fn get   (&self, req: Request) -> Response { res("get"   ) }
        fn post  (&self, req: Request) -> Response { res("post"  ) }
        fn patch (&self, req: Request) -> Response { res("patch" ) }
        fn put   (&self, req: Request) -> Response { res("put"   ) }
    }
    #[test]
    fn test_preroute() {
        let api = TestApi::new();
        assert_eq!(format!("{:?}", api.preroute(req(Method::Get))),
            format!("{:?}", RouteHint::CallMe(req(Method::Get))));
        assert_eq!(format!("{:?}", api.preroute(bad_req())),
            format!("{:?}", RouteHint::NotMe(bad_req())));
    }
    #[test]
    fn test_route() {
        fn ast(api: &TestApi, req: &Request, txt: &str) {
            let left = api.route(req.clone());
            let right = res(txt);
            assert_eq!(format!("{:?}", left), format!("{:?}", right));
        }
        let api = TestApi::new();
        ast(&api, &req(Method::Delete), "delete");
        ast(&api, &req(Method::Get), "get");
        ast(&api, &req(Method::Post), "post");
        ast(&api, &req(Method::Patch), "patch");
        ast(&api, &req(Method::Put), "put");
    }
    #[test]
    fn test_postroute() {
        let api = TestApi::new();
        assert_eq!(format!("{:?}", api.postroute(res(""))), format!("{:?}", res("")));
    }
}
