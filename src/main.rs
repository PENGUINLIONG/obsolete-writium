extern crate markdown;
extern crate iron;

use std::io::{Read};
use std::fs::{File};
use std::path::Path;
use iron::prelude::*;
use iron::status;
use iron::method::Method;

mod settings;

/// Response error page if it exists. otherwise, response with error code.
fn gen_error_page(code: status::Status) -> Response {
    let err_code_literal = match code {
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
    };
    println!("Generating error page from status {}.", err_code_literal);
    let file_name = settings::ERROR_DIR.to_owned() + err_code_literal + ".html";
    if let Ok(mut file) = File::open(file_name) {
        let mut err_page = String::new();
        if file.read_to_string(&mut err_page).is_err() {
            // Cannot read error page file.
            return Response::with((code));
        }
        // Successfully read.
        return Response::with((code, err_page));
    } else {
        // Cannot open error page file. Response with the error code.
        return Response::with((code, err_code_literal));
    }
}

/// Find article from local storage.
///
/// Some(~) will be returned if the requested article is successfully read.
/// None, otherwise.
fn find_article(path: &Vec<&str>) -> Option<String> {
    let path = settings::POST_DIR.to_owned() + &path.join("/") + ".md";
    println!("Looking for file in local storage: {}", path);
    let path = Path::new(&path);
    if let Ok(mut file) = File::open(&path) {
        // Fetch article.
        let mut article = String::new();
        if file.read_to_string(&mut article).is_err() {
             return None;
        }
        // Translate markdown.
        let article = markdown::to_html(&article);
        return Some(article);
    } else {
        return None;
    }
}

/// Response to incoming requests.
fn response(req: &mut Request) -> IronResult<Response> {
    // Only GET method is allowed.
    if req.method != Method::Get {
        return Ok(gen_error_page(status::MethodNotAllowed));
    }
    let path = req.url.path();
    println!("Url of incoming request: {}", req.url);
    return Ok(match find_article(&path) {
        Some(article) => Response::with((status::Ok, article)),
        None => gen_error_page(status::NotFound),
    });
}

fn main() {
    Iron::new(response).http(settings::HOST_ADDR).unwrap();
}
