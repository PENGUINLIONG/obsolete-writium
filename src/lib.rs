extern crate futures;
pub extern crate hyper;
#[macro_use]
extern crate log;
pub extern crate serde_json;

// Writium.
mod writium;

pub use writium::Writium;

// Api and namespace.
mod api;
mod callback;
mod namespace;

pub use api::{Api, ApiFuture, ApiResult, RouteHint};
pub use callback::Callback;
pub use namespace::Namespace;

// Request flow protocol.
mod proto;

pub use proto::{HyperBody, HyperRequest, HyperResponse, Request, RequestBody, Response};

// Error handling.
mod error;
mod result;

pub use error::WritiumError;
pub use result::{ok, err, FutureExt, WritiumFuture, WritiumResult};
