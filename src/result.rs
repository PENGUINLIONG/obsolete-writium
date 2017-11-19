use futures::future::{Future};
use super::WritiumError;

pub type WritiumFuture<T> = Future<Item=T, Error=WritiumError>;
pub type WritiumResult<T> = Box<WritiumFuture<T>>;

pub fn ok<T: 'static>(t: T) -> WritiumResult<T> {
    Box::new(::futures::future::ok(t))
}
pub fn err<T: 'static>(e: WritiumError) -> WritiumResult<T> {
    Box::new(::futures::future::err(e))
}

pub trait FutureExt: Future + Sized {
    fn into_box(self) -> Box<Future<Item = Self::Item, Error = Self::Error>>;
}

impl<F: Future + 'static> FutureExt for F {
    fn into_box(self) -> Box<Future<Item = Self::Item, Error = Self::Error>> {
        Box::new(self)
    }
}