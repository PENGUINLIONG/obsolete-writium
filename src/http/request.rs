use std::io::Read;
use std::net::SocketAddr;
use ::serde_json as json;
use ::iron::Request as IronRequest;

pub use ::iron::method::Method;
pub type Path = Vec<String>;
pub type Queries = Vec<(String, String)>;
pub type Json = ::serde_json::Value;

fn body_to_json(body: &mut ::iron::request::Body) -> Option<json::Value> {
    let mut json = String::new();
    if let Err(_) = body.read_to_string(&mut json) {
        warn!("Unable to read body.");
        return None
    }
    match json::from_str::<json::Value>(&json) {
        Ok(j) => Some(j),
        Err(_) => {
            warn!("Unable to parse body as json.");
            None
        },
    }
}

#[derive(Clone, Debug)]
pub struct Request {
    method: Method,
    remote_addr: Option<SocketAddr>,
    path: Path,
    queries: Queries,
    content: Option<Json>,
}
impl Request {
    /// Create a new request object.
    pub fn new(method: Method) -> Request {
        Request {
            method: method,
            remote_addr: None,
            path: Vec::new(),
            queries: Vec::new(),
            content: None,
        }
    }
    #[inline]
    pub fn method(&self) -> &Method {
        &self.method
    }
    #[inline]
    pub fn method_mut(&mut self) -> &mut Method {
        &mut self.method
    }
    #[inline]
    pub fn remote_addr(&self) -> Option<&SocketAddr> {
        self.remote_addr.as_ref()
    }
    #[inline]
    pub fn remote_addr_mut(&mut self) -> Option<&mut SocketAddr> {
        self.remote_addr.as_mut()
    }
    #[inline]
    pub fn path(&self) -> &Path {
        &self.path
    }
    #[inline]
    pub fn path_mut(&mut self) -> &mut Path {
        &mut self.path
    }
    #[inline]
    pub fn queries(&self) -> &Queries {
        &self.queries
    }
    #[inline]
    pub fn queries_mut(&mut self) -> &mut Queries {
        &mut self.queries
    }
    #[inline]
    pub fn content(&self) -> Option<&Json> {
        self.content.as_ref()
    }
    #[inline]
    pub fn content_mut(&mut self) -> Option<&mut Json> {
        self.content.as_mut()
    }
    

    pub fn from_iron_request(req: &mut IronRequest) -> Request {
        let url = req.url.as_ref();
        // Iron borrow these stuff by mutable reference.
        let mut rv = Request {
            method: req.method.clone(),
            remote_addr: Some(req.remote_addr),
            path: Vec::new(),
            queries: Vec::new(),
            content: if req.method == Method::Get || req.method == Method::Delete {
                    body_to_json(&mut req.body)
                } else {
                    None
                },
        };
        if let Some(pth) = url.path_segments() {
            rv.path.extend(pth.map(|x| x.into()));
        }
        rv.queries.extend(url.query_pairs().map(|(x, y)| (x.into_owned(), y.into_owned()) ));
        rv
    }
}
impl Into<Option<Json>> for Request {
    fn into(self) -> Option<Json> {
        self.content
    }
}
