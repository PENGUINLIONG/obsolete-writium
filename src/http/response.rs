use ::http::status::Status;

/// Response type for all API calls.
///
/// `Sucess` carries the result of an API call with or without any recoverable
/// error. Recoverable errors are errors can be recovered from, using default
/// settings; or can be replaced with API's default output.
///
/// `Failed` means that an API call was failed with an unrecoverable error
/// generated. For instance, the request URL was in an unrecognizable pattern;
/// or an local file requested was used but not in shared mode. The second term
/// is a short description of critical information about the failure. The
/// description will be logged on warning level.
pub enum Response {
    Done(Status, String),
    Failed(Status, &'static str),
}
