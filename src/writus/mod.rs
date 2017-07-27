extern crate markdown;
extern crate iron;

use std::fs::{File};
use std::io::{Read};
use std::path::Path;

use self::iron::prelude::*;
use self::iron::method::Method;
use self::iron::status;

mod response_gen;
mod settings;

use self::response_gen::{gen_error, gen_error_page, gen_page, gen_spec};

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

fn make_response_rsc(local_dir: &str, rel_path: &str, ext: &str) -> Response {
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
    match map_ext(ext) {
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
}

fn make_response_article(local_dir: &str, rel_path: &str) -> Response {
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
    let rel_path = rel_path.to_owned() + ".md";
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
}

/// Response to incoming requests.
fn make_response(req: &Request) -> Response {
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
    let rel_path = "/".to_owned() + &path[1..].join("/");
    if let Some(ext_pos) = rel_path.rfind(".") {
        // '.' was found, there is an extension. Look up supported file formats.
        make_response_rsc(&local_dir, &rel_path, &rel_path[(ext_pos + 1)..])
    } else if search_dir == "post" {
        make_response_article(&local_dir, &rel_path)
    } else {
        println!("Article can only be in `./post`.");
        gen_error_page(status::NotFound)
    }
}

fn response(req: &mut Request) -> IronResult<Response> {
    println!("-- Response Begin --");
    let res = Ok(make_response(&req));
    println!("-- Response End --");
    return res;
}

pub fn start() {
    Iron::new(response).http(settings::HOST_ADDR).unwrap();
}