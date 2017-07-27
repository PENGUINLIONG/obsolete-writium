extern crate markdown;
extern crate iron;

use std::fs::{File};
use std::io::{Read};
use std::path::Path;

use iron::prelude::*;
use iron::headers::{ContentType};
use iron::method::Method;
use iron::status;

mod settings;

/// Response normal web page with given HTML data.
fn gen_page(html: String) -> Response {
    let mut res = Response::with((status::Ok, html));
    res.headers.set(ContentType::html());
    res
}
/// Response materials of special types.
fn gen_spec(data: Vec<u8>, content_type: String) -> Response {
    let mut res = Response::with((status::Ok, data));
    
    res.headers.set_raw("Content-Type", vec![content_type.into_bytes()]);
    res
}


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
/// Response error code simply.
fn gen_error(code: status::Status) -> Response {
    let err_code_literal = map_error_code(code);
    println!("Generating error from status {}.", err_code_literal);    
    Response::with((code))
}
/// Response error page if it exists. otherwise, response with error code.
fn gen_error_page(code: status::Status) -> Response {
    let err_code_literal = map_error_code(code);
    println!("Generating error page from status {}.", err_code_literal);
    // Don't use `find_rsc`. It will loop forever.
    let file_name =
        settings::ERROR_DIR.to_owned() + "/" + &err_code_literal + ".html";
    match File::open(file_name) {
        Ok(mut file) => {
            let mut err_page = String::new();
            match file.read_to_string(&mut err_page) {
                // Successfully read.
                Ok(_) => {
                    let mut res = Response::with((code, err_page));
                    res.headers.set(ContentType::html());
                    res
                },
                // Cannot read error page file.
                Err(_) => gen_error(code),
            }
        },
        // Cannot open error page file. Response with the error code.
        Err(_) => gen_error(code),
    }
}

/// Find resource from local storage.
///
/// Some(~) will be returned if the requested resource is successfully read.
/// None, otherwise.
fn find_rsc(local_dir: &str, path: &str) -> Option<Vec<u8>> {
    let local_path = local_dir.to_owned() + path;
    println!("Looking for file in local storage: {}", local_path);
    let local_path = Path::new(&local_path);
    if let Ok(mut file) = File::open(&local_path) {
        // Fetch content.
        let mut content = Vec::<u8>::new();
        if file.read_to_end(&mut content).is_err() {
            return None;
        }
        return Some(content);
    } else {
        return None;
    }
}

/// Response to incoming requests.
fn response(req: &Request) -> Response {
    /// Translate markdown byte sequence to HTML.
    fn translate_article(bytes: Vec<u8>) -> Option<String> {
        match String::from_utf8(bytes) {
            Ok(content) => {
                println!("Translate Markdown into HTML.");
                Some(markdown::to_html(&content))
            },
            Err(_) => None,
        }
    }
    /// Map file extension to media type.
    fn map_ext(ext: &str) -> Option<&str> {
        match ext {
            // General.
            "htm" => Some("text/html"),
            "html" => Some("text/html"),
            "js" => Some("application/javascript"),
            "css" => Some("text/css"),
            // Image.
            "jpg" => Some("image/jpeg"),
            "jpeg" => Some("image/jpeg"),
            "png" => Some("image/png"),
            "gif" => Some("image/gif"),
            _ => None,
        }
    }
    /// Map search directory to local storage directory.
    fn map_search_dir(search_dir: &str) -> Option<String> {
        match search_dir {
            "post" => Some(settings::POST_DIR.to_owned()),
            "static" => Some(settings::STATIC_DIR.to_owned()),
            // "error", "template" => No, these are not directly exposed.
            _ => None,
        }
    }
    // Only GET method is allowed.
    if req.method != Method::Get {
        println!("Invalid HTTP method.");
        return gen_error_page(status::MethodNotAllowed);
    }
    let path = req.url.path();
    println!("Url of incoming request: {}", req.url);
    // Assign different search directory for different root. If the requested
    // thing doesn't exist, ignore with 404 returned.
    let search_dir = path.get(0).unwrap().to_owned();
    if search_dir == "" {
        println!("Empty search directory, there will be a index page in the future.");
        return gen_error_page(status::NotFound);
    }
    println!("Search directory is {}.", search_dir);
    let local_dir = match map_search_dir(&search_dir) {
        Some(dir) => dir,
        None => {
            println!("Search directory not exposed.");
            return gen_error_page(status::NotFound);
        },
    };
    // Read data from storage.
    // The last section of $path contains the file name and its extension, which
    // indicates its file format.
    let file_name = path.last().unwrap();
    let mut rel_path = "/".to_owned() + &path[1..].join("/");
    if let Some(ext_pos) = file_name.rfind(".") {
        // '.' was found, there is an extension. Look up supported file formats.
        match map_ext(&file_name[(ext_pos + 1)..]) {
            Some(content_type) => match find_rsc(&local_dir, &rel_path) {
                Some(rsc) => gen_spec(rsc, content_type.to_string()),
                None => {
                    println!("Resource not found.");
                    gen_error(status::NotFound)
                },
            },
            None => {
                println!("Unrecognized file extension.");
                gen_error(status::NotFound)
            },
        }
    } else if search_dir == "post" {
        rel_path += ".md";
        // Not found, it should be an article.
        match find_rsc(&local_dir, &rel_path) {
            // Article present, translate markdown to HTML.
            Some(content) => match translate_article(content) {
                Some(html) => gen_page(html),
                // Treat translation error as file-not-found error.
                None => {
                    println!("Markdown translation failed.");
                    gen_error_page(status::NotFound)
                },
            },
            None => {
                println!("Article absent.");
                gen_error_page(status::NotFound)
            },
        }
    } else {
        println!("Article can only be in `./post`.");
        gen_error_page(status::NotFound)
    }
}

fn response_wrapper(req: &mut Request) -> IronResult<Response> {
    println!("-- Response Begin --");
    let res = Ok(response(&req));
    println!("-- Response End --");
    return res;
}

fn main() {
    Iron::new(response_wrapper).http(settings::HOST_ADDR).unwrap();
}
