extern crate iron;

use self::iron::prelude::*;
use self::iron::method::Method;
use self::iron::status;

mod cache;
mod resource;
mod response_gen;
mod settings;
mod template;

use self::response_gen::{gen_error, gen_error_page, gen_page, gen_spec,
    gen_redirection};

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
    // $path is guaranteed to have at least 1 element.
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
    use self::resource::Resource::{Article, InvalidArticle,
        Material, InvalidMaterial, AddSlash};
    let path = "/".to_owned() + &path[1..].join("/");
    let local_path = local_dir + &path;
    match resource::get_resource(&local_path, search_dir == "post") {
        Some(rsc) => match rsc {
            Article { content } => gen_page(content),
            InvalidArticle => gen_error_page(status::NotFound),
            Material { media_type, data } => gen_spec(data, media_type),
            InvalidMaterial => gen_error(status::NotFound),
            AddSlash => gen_redirection(&(format!("/{}{}/", &search_dir, &path))),
        },
        None => gen_error_page(status::NotFound),
    }
}

fn response(req: &mut Request) -> IronResult<Response> {
    println!("-- Response Begin --");
    let res = Ok(make_response(&req));
    println!("-- Response End --");
    return res;
}

pub fn start() {
    let ca = cache::Cache::new();
    ca.gen_cache();
    Iron::new(response).http(settings::HOST_ADDR).unwrap();
}
