use futures::Future;
use hyper::{Headers, Method, Uri};
use hyper::header::Header;

pub use hyper::Request as HyperRequest;

fn collect_path_segs(path: &str) -> Option<Vec<String>> {
    let mut raw_rv = Vec::new();
    if path.as_bytes()[0] == b'/' {
        raw_rv.extend(path[1..].split('/').map(|x| x.to_string()))
    }
    // Prevent protection for path traversal attack.
    let mut rv = Vec::new();
    for seg in raw_rv {
        if seg == ".." {
            if rv.pop().is_none() {
                return None
            }
        } else if seg == "." {
        } else {
            rv.push(seg);
        }
    }
    Some(rv)
}

pub struct Request {
    method: Method,
    uri: Uri,
    path_segs: Vec<String>,
    headers: Headers,
    body: Box<Future<Item=::hyper::Chunk, Error=::hyper::Error>>,
}
impl Request {
    pub fn new(method: Method, uri: &str) -> Option<Request> {
        use std::str::FromStr;
        let uri = if let Ok(u) = Uri::from_str(uri) { u }
            else { return None };
        Some(Request {
            path_segs:
                if let Some(segs) = collect_path_segs(uri.path()) { segs }
                else { return None },
            method: method,
            uri: uri,
            headers: Headers::new(),
            body: Box::new(::futures::future::ok([].as_ref().into())),
        })
    }
    pub fn construct(method: Method, uri: Uri, headers: Headers,
        body: Box<Future<Item=::hyper::Chunk, Error=::hyper::Error>>)
        -> Option<Request> {
        Some(Request {
            path_segs:
                if let Some(segs) = collect_path_segs(uri.path()) { segs }
                else { return None },
            method: method,
            uri: uri,
            headers: headers,
            body: body,
        })
    }

    pub fn with_header<H: Header>(mut self, header: H) -> Self {
        self.headers.set::<H>(header);
        self
    }
    pub fn with_headers(mut self, headers: Headers) -> Self {
        self.headers = headers;
        self
    }
    pub fn with_body<T>(mut self, body: T) -> Self
        where ::hyper::Chunk: From<T> {
        self.body = Box::new(::futures::future::ok(body.into()));
        self
    }

    pub fn method(&self) -> &Method {
        &self.method
    }
    pub fn uri(&self) -> &Uri {
        &self.uri
    }
    pub fn headers(&self) -> &Headers {
        &self.headers
    }
    pub fn body(self) -> Box<Future<Item=::hyper::Chunk, Error=::hyper::Error>>{
        self.body
    }

    /// Get the reference to internal path segment record.
    pub fn path_segs(&self) -> &[String] {
        &self.path_segs[..]
    }

    /// Route a segment of path. If the incoming segment is about the current
    /// request, the segment is removed from internal record and is then
    /// returned. Otherwise, `None` is returned. If the `seg` parameter is
    /// `None`, this method will always succeed except for when there is no segment to be
    /// routed.
    pub fn route_seg(&mut self, seg: Option<&str>) -> Option<String> {
        if self.path_segs.len() > 0 &&
            (seg.is_none() || self.path_segs[0] == seg.unwrap()) {
                Some(self.path_segs.remove(0))
        } else {
            None
        }
    }
}
