use std::io;
use std::sync::{Arc, Condvar, Mutex};
use std::thread::Builder;

use writus::iron::prelude::*;
use writus::iron::method::Method;
use writus::iron::status;

use writus::cache::Cache;
use writus::resource;
use writus::response_gen::{gen_error, gen_error_page, gen_page, gen_spec,
    gen_redirection};
use writus::settings;

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
    use writus::resource::Resource::{Article, InvalidArticle,
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

pub struct Server {
    cache: Cache,
    cvar: Arc<(Mutex<()>, Condvar)>,
}

impl Server {
    pub fn new() -> Server {
        let arc_held_by_close = Arc::new((Mutex::new(()), Condvar::new()));
        let arc_held_by_server = arc_held_by_close.clone();
        // TODO: When to join?
        let join_handle = Builder::new()
            .name("iron".to_owned())
            .spawn(move|| {
                let mut serv = Iron::new(response)
                    .http(settings::HOST_ADDR)
                    .unwrap();
                let &(ref dumb, ref cvar) = &*arc_held_by_server;
                if let Err(_) = cvar.wait(dumb.lock().unwrap()) {
                    println!("Failed to wait for close signal. Going to close now.");
                }
                if let Err(_) = serv.close() {
                    println!("Failed to close iron.");
                }
        });
        // Start cache system.
        let ca = Cache::new();
        ca.gen_cache();

        Server {
            cache: ca,
            cvar: arc_held_by_close,
        }
    }

    fn close(&self) {
        let &(ref dumb, ref cvar) = &*self.cvar;
        cvar.notify_one();
    }

    fn interpret_command(&mut self, command: &str, args: &[&str]) -> bool {
        match command {
            "close" => {
                self.close();
                return true;
            },
            "recache" => {
                self.cache = Cache::new();
                self.cache.gen_cache();
            },
            _ => println!("Unknown command."),
        }
        false
    }

    pub fn process_commands(&mut self) {
        let input = io::stdin();
        loop {
            let mut line = String::new();
            if let Err(_) = input.read_line(&mut line) { break }
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

