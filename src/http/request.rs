// TODO: Add percent code decoding.
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

pub struct Request {
    method: Method,
    remote_addr: SocketAddr,
    path: Path,
    queries: Queries,
    content: Option<Json>,
}
impl Request {
    #[inline]
    pub fn method(&self) -> Method {
        self.method.clone()
    }
    #[inline]
    pub fn remote_addr(&self) -> SocketAddr {
        self.remote_addr.clone()
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
    fn collect_queries(&mut self, query: Option<&str>) {
        if let Some(query) = query {
            let pairs = query[1..].split('&');
            for pair in pairs {
                let mut key_val = pair.split('=');
                // Ignore invalid pair.
                let key = if let Some(key) = key_val.next() { key }
                    else { continue };
                let val = if let Some(val) = key_val.next() { val }
                    else { continue };
                if key_val.next().is_some() { continue }
                self.queries.push((key.to_string(), val.to_string()));
            }
        }
    }
    fn collect_path_segments(&mut self, path: &str) {
        self.path = if path.starts_with('/') {
            path[1..].split('/')
                .map(|x| x.to_string())
                .collect()
        } else {
            Vec::<String>::new()
        }
    }

    pub fn from_iron_request(req: &mut IronRequest) -> Request {
        let url = req.url.as_ref();
        // Iron borrow these stuff by mutable reference.
        let mut rv = Request {
            method: req.method.clone(),
            remote_addr: req.remote_addr,
            path: Vec::new(),
            queries: Vec::new(),
            content: if req.method == Method::Get || req.method == Method::Delete {
                    body_to_json(&mut req.body)
                } else {
                    None
                },
        };
        rv.collect_path_segments(url.path());
        rv.collect_queries(url.query());
        rv
    }
}
impl Into<Option<Json>> for Request {
    fn into(self) -> Option<Json> {
        self.content
    }
}
