///! This module includes most of the tools you want to use *during API*
///! *development*. In case you are a user consuming APIs made by others, you
///! generally need `Writium` only.

// Api and namespace implementation use.
pub use api::{Api, ApiName, ApiResult};
pub use namespace::Namespace;
pub use callback::Callback;

// Request and response.
pub use proto::{Request, Response};

// Error handling.
pub use error::WritiumError;
pub use result::{ok, err, FutureExt, WritiumResult};
