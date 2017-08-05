extern crate chrono;
extern crate iron;
extern crate json;
extern crate getopts;
extern crate markdown;

use std::io;
use std::path::{Path, PathBuf};

use self::iron::prelude::*;
use self::iron::method::Method;
use self::iron::status;

mod resource;
mod response_gen;
mod template;

pub mod settings;

use self::settings::CONFIGS;
use self::response_gen::{gen_error, gen_error_page, gen_page, gen_spec,
    gen_redirection};

/// Response to incoming requests.
fn make_response(req: &Request) -> Response {
    /// Map search directory to local storage directory.
    fn map_search_dir(search_dir: &str) -> Option<&Path> {
        match search_dir {
            "post" => Some(Path::new(&CONFIGS.post_dir)),
            "static" => Some(Path::new(&CONFIGS.static_dir)),
            // "error", "template" => No, these are not directly exposed.
            _ => None,
        }
    }
    // Only GET method is allowed.
    if req.method != Method::Get {
        info!("Invalid HTTP method.");
        return gen_error(status::MethodNotAllowed);
    }
    // $path is guaranteed to have at least 1 element.
    let path = req.url.path();
    info!("Url of incoming request: {}", req.url);
    // Assign different search directory for different root. If the requested
    // thing doesn't exist, ignore with 404 returned.
    let search_dir = path.get(0).unwrap().to_owned();
    if search_dir == "" {
        info!("Empty search directory, there will be a index page in the future.");
        return gen_error_page(status::NotFound);
    }
    info!("Search directory is {}.", search_dir);
    let local_dir = match map_search_dir(&search_dir) {
        Some(dir) => dir,
        None => {
            info!("Search directory not exposed.");
            return gen_error_page(status::Forbidden);
        },
    };
    // Read data from storage.
    use writus::resource::Resource::{Article, InvalidArticle,
        Material, InvalidMaterial, AddSlash};
    let path = &path[1..].join("/");
    let mut local_path = PathBuf::from(&local_dir);
    local_path.push(&path);
    // Make sure requested file is under published directory.
    match local_path.canonicalize() {
        Ok(buf) =>
            // Canonicalize $local_dir because the annoying prefix `\\?\` on
            // Windows.
            if !buf.starts_with(Path::new(&local_dir).canonicalize().unwrap()) {
            // Even you access a file in a published directory from another one
            // will lead to this error.
            info!("Requested resource is outside of published directory.");
            return gen_error_page(status::Forbidden);
        },
        Err(_) => {
            info!("Resource cannot be located.");
            return gen_error_page(status::NotFound);
        }
    }
    // Get resource.
    match resource::get_resource(local_path.as_path(), search_dir == "post") {
        Some(rsc) => match rsc {
            Article { content } => gen_page(content),
            InvalidArticle => gen_error_page(status::NotFound),
            Material { media_type, data } => gen_spec(data, media_type),
            InvalidMaterial => gen_error(status::NotFound),
            AddSlash => gen_redirection(&(format!("/{}/{}/", &search_dir, &path))),
        },
        None => gen_error_page(status::NotFound),
    }
}

fn response(req: &mut Request) -> IronResult<Response> {
    info!("-- Response Begin --");
    let res = Ok(make_response(&req));
    info!("-- Response End --");
    return res;
}

struct _Writus {
    close_iron: Box<FnMut()>,
}

pub struct Writus {
    _writus: _Writus,
}
impl Writus {
    pub fn new() -> Writus {
        let mut iron = Iron::new(response).http(&CONFIGS.host_addr).unwrap();
        resource::gen_cache();
        Writus {
            _writus: _Writus {
                close_iron: Box::new(move || { let _ = iron.close(); }),
            },
        }
    }

    fn interpret_command(&mut self, command: &str, args: &[&str]) -> bool {
        match command {
            "close" => {
                (self._writus.close_iron)();
                return true;
            },
            "recache" => {
                resource::remove_cache();
                resource::gen_cache();
            },
            _ => error!("Unknown command."),
        }
        false
    }

    pub fn process_commands(&mut self) {
        let input = io::stdin();
        loop {
            let mut line = String::new();
            if let Err(_) = input.read_line(&mut line) { break }
            info!("Received command: {}", &(line).trim());
            let parts: Vec<&str> = line.split_whitespace().collect();
            let need_exit = match parts.len() {
                0 => self.interpret_command("", &vec![""]),
                1 => self.interpret_command(parts[0], &vec![""]),
                _ => self.interpret_command(parts[0], &parts[1..]),
            };

            if need_exit { break }
        }
    }
}
impl Drop for Writus {
     fn drop(&mut self) {
        resource::remove_cache();
        (self._writus.close_iron)();
     }
}
