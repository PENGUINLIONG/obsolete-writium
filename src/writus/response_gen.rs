use std::path::PathBuf;

use writus::iron::prelude::*;
use writus::iron::headers::{ContentType};
use writus::iron::status;

use writus::resource;
use writus::settings;

/// Map error code to error literal.
fn map_error_code(code: status::Status) -> String {
    match code {
        status::BadRequest => "400",
        status::Unauthorized => "401",
        status::PaymentRequired => "402",
        status::Forbidden => "403",
        status::NotFound => "404",
        status::MethodNotAllowed => "405",
        status::NotAcceptable => "406",
        status::ProxyAuthenticationRequired => "407",
        status::RequestTimeout => "408",
        status::Conflict => "409",
        status::Gone => "410",
        status::LengthRequired => "411",
        status::PreconditionFailed => "412",
        status::PayloadTooLarge => "413",
        status::UriTooLong => "414",
        status::UnsupportedMediaType => "415",
        status::RangeNotSatisfiable => "416",
        status::ExpectationFailed => "417",
        status::ImATeapot => "418",
        status::MisdirectedRequest => "421",
        status::UnprocessableEntity => "422",
        status::Locked => "423",
        status::FailedDependency => "424",
        status::UpgradeRequired => "426",
        status::PreconditionRequired => "428",
        status::TooManyRequests => "429",
        status::RequestHeaderFieldsTooLarge => "431",
        status::UnavailableForLegalReasons => "451",
        _ => "Unknown",
    }.to_owned()
}

/// Response normal web page with given HTML data.
pub fn gen_page(html: String) -> Response {
    let mut res = Response::with((status::Ok, html));
    res.headers.set(ContentType::html());
    res
}
/// Response materials of special types.
pub fn gen_spec(data: Vec<u8>, content_type: String) -> Response {
    let mut res = Response::with((status::Ok, data));
    
    res.headers.set_raw("Content-Type", vec![content_type.into_bytes()]);
    res
}
/// Response error code simply.
pub fn gen_error(code: status::Status) -> Response {
    let err_code_literal = map_error_code(code);
    info!("Generating error from status {}.", err_code_literal);    
    Response::with((code))
}
/// Response error page if it exists. otherwise, response with error code.
pub fn gen_error_page(code: status::Status) -> Response {
    let err_code_literal = map_error_code(code);
    info!("Generating error page from status {}.", &err_code_literal);
    // Don't use `find_rsc`. It will loop forever.
    let mut path = PathBuf::new();
    path.push(settings::ERROR_DIR);
    path.push(err_code_literal + ".html");
    match resource::load_text_resource(path.as_path()) {
        Some(s) => {
            let mut res = Response::with((code, s));
            res.headers.set(ContentType::html());
            res
        },
        None => gen_error(code),
    }
}
/// Response redirection.
pub fn gen_redirection(location: &str) -> Response {
    info!("Generating Rediretion to: {}", location);
    let mut res = Response::with((status::Found));
    res.headers.set_raw("Location", vec![location.to_owned().into_bytes()]);
    res
}
