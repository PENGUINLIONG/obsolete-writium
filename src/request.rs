use std::borrow::{Borrow, BorrowMut};
use std::ops::{Deref, DerefMut};
use super::hyper::{Body, Uri};

pub use hyper::Request as HyperRequest;

fn collect_path_segs(uri: &Uri) -> Option<Vec<String>> {
    let path = uri.path();
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
    inner: HyperRequest,
    path_segs: Vec<String>,
}
impl Request {
    pub fn new(req: HyperRequest) -> Option<Request> {
        if let Some(segs) = collect_path_segs(req.uri()) {
            Some(Request {
                path_segs: segs,
                inner: req,
            })
        } else {
            None
        }
    }
    pub fn body(self) -> Body {
        self.inner.body()
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
impl Into<HyperRequest> for Request {
    fn into(self) -> HyperRequest {
        self.inner
    }
}
impl Into<::hyper::Body> for Request {
    fn into(self) -> ::hyper::Body {
        self.inner.body()
    }
}
impl AsRef<HyperRequest> for Request {
    fn as_ref(&self) -> &HyperRequest {
        &self.inner
    }
}
impl AsMut<HyperRequest> for Request {
    fn as_mut(&mut self) -> &mut HyperRequest {
        &mut self.inner
    }
}
impl Borrow<HyperRequest> for Request {
    fn borrow(&self) -> &HyperRequest {
        &self.inner
    }
}
impl BorrowMut<HyperRequest> for Request {
    fn borrow_mut(&mut self) -> &mut HyperRequest {
        &mut self.inner
    }
}
impl Deref for Request {
    type Target = HyperRequest;
    fn deref(&self) -> &HyperRequest {
        &self.inner
    }
}
impl DerefMut for Request {
    fn deref_mut(&mut self) -> &mut HyperRequest {
        &mut self.inner
    }
}
