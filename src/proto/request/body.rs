use futures::{Future, Poll, Stream};
use error::WritiumError;
use hyper::StatusCode;
use futures::future::ok;

pub use ::hyper::Body as HyperBody;

pub struct RequestBody(Box<Future<Item=Vec<u8>, Error=WritiumError>>);
impl RequestBody {
    fn new() -> RequestBody{
        RequestBody(Box::new(ok(Vec::new())))
    }
}
impl From<HyperBody> for RequestBody {
    fn from(body: HyperBody) -> RequestBody {
        RequestBody(Box::new(body.concat2()
            .map(|x| x.to_owned())
            .map_err(|_| WritiumError::new(StatusCode::BadRequest, "Unable to retrieve content."))
        ))
    }
}
impl From<&'static [u8]> for RequestBody {
    fn from(bytes: &'static [u8]) -> RequestBody {
        RequestBody(Box::new(ok(bytes.to_owned())))
    }
}
impl From<Vec<u8>> for RequestBody {
    fn from(bytes: Vec<u8>) -> RequestBody {
        RequestBody(Box::new(ok(bytes)))
    }
}
impl From<&'static str> for RequestBody {
    fn from(string: &'static str) -> RequestBody {
        RequestBody(Box::new(ok(string.as_bytes().to_owned())))
    }
}
impl From<String> for RequestBody {
    fn from(string: String) -> RequestBody {
        RequestBody(Box::new(ok(string.as_bytes().to_owned())))
    }
}
impl Default for RequestBody {
    fn default() -> RequestBody {
        RequestBody(Box::new(ok(Vec::new())))
    }
}

impl Future for RequestBody {
    type Item = Vec<u8>;
    type Error = WritiumError;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.0.poll()
    }
}
