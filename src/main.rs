extern crate markdown;
extern crate iron;

use std::io::{Read};
use std::fs::{File};
use std::path::Path;
use iron::prelude::*;
use iron::status;
use iron::method::Method;

const LOCAL_STORAGE_CONFIG: &str = "F:/Articles/";

/// Find article from local storage.
///
/// Some(~) will be returned if the requested article is successfully read.
/// None, otherwise. 
fn find_article(path: &Vec<&str>) -> Option<String> {
    let path = String::from(LOCAL_STORAGE_CONFIG) + &path.join("/") + ".md";
    println!("Looking for file in local storage: {}", path);
    let path = Path::new(&path);
    if let Ok(mut file) = File::open(&path) {
        // Fetch 
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
        return Ok(Response::with((status::MethodNotAllowed)))
    }
    // Last part of the path is going to be the file name.
    let path = req.url.path();
    println!("Url of incoming request: {}", req.url);
    if let Some(article) = find_article(&path) {
        return Ok(Response::with((status::Ok, article)));
    } else {
        return Ok(Response::with((status::NotFound)));
    }
}

fn main() {
    Iron::new(response).http("localhost:23333").unwrap();
}