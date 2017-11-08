// Web service.
extern crate hyper_native_tls;
extern crate iron;
extern crate url;
// Serde.
extern crate serde;
extern crate serde_json;
// Logging.
#[macro_use]
extern crate log;

pub mod writium;
pub use writium::Writium;
pub mod api;
pub mod http;

mod prelude {
    pub use writium::Writium;
    pub use api::{Api, Cache, Namespace};
    pub use http::{Request, Response};
}
