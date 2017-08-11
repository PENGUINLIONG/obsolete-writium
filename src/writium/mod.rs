extern crate chrono;
extern crate iron;
extern crate json;
extern crate getopts;
extern crate hyper_native_tls;
extern crate markdown;
extern crate url;

use std::io;
use std::path::Path;
use std::sync::{Arc, RwLock};

use self::iron::prelude::*;
use self::iron::method::Method;
use self::iron::status;

use self::hyper_native_tls::NativeTlsServer;

mod resource;
mod response_gen;
mod template;

pub mod settings;

use self::settings::CONFIGS;
use self::resource::Resource;
use self::resource::Resource::*;
use self::response_gen::{gen_error, gen_error_page, gen_page, gen_spec,
    gen_redirection};

fn resource_to_response(path: &str, resource: Option<Resource>) -> Response {
    match resource {
        Some(rsc) => match rsc {
            Article { content } => gen_page(content),
            InvalidArticle => gen_error_page(status::NotFound),
            Material { media_type, data } => gen_spec(data, media_type),
            InvalidMaterial => gen_error(status::NotFound),
            AddSlash => gen_redirection(&(format!("/{}/", &path))),
        },
        None => gen_error_page(status::NotFound),
    }
}

/// Shared data object carrying all the informations might be used to make
/// response.
struct WritiumServer {
    /// Map of listed articles sorted by publish time.
    cached_articles: resource::CachedArticles,
}
impl WritiumServer {
    /// Make response for non-root directories. Only `./post` is allowed to
    /// store articles. Requests for articles out of it will be responded with
    /// 404.
    fn make_response_for_dir(&self, local_dir: String, path: String,
        in_post_dir: bool) -> Response {
        // Access to directory-root is not allowed.
        if path.is_empty() { return gen_error_page(status::Forbidden); }
        let local_path = path_buf![&local_dir, &path];
        // Make sure requested file is under published directory.
        match local_path.canonicalize() {
            // Canonicalize $local_dir because the annoying prefix `\\?\` on
            // Windows.
            Ok(buf) => if !buf.starts_with(
                Path::new(&local_dir).canonicalize().unwrap()
            ) {
                // Even you access a file in a published directory from
                // another one will lead to this error.
                info!("Requested resource is outside of published directory.");
                return gen_error_page(status::Forbidden);
            },
            Err(_) => {
                info!("Resource cannot be located.");
                return gen_error_page(status::NotFound);
            }
        }
        resource_to_response(
            &path,
            resource::get_resource(local_path.as_path(), in_post_dir)
        )
    }    
    /// Make response for root directory.
    fn make_response_for_root(&self, path: String,
        query: Option<&str>) -> Response {
        if path.is_empty() {
            // Index page.
            info!("Request for index.");
            resource_to_response(
                &path,
                if let Some(q) = query {
                    let mut page: u32 = 0;
                    for pair in q.split('&') {
                        let mut key_n_val = pair.split('=');
                        let key = key_n_val.next();
                        let val = key_n_val.next();
                        if key.is_some() && key.unwrap() == "page" &&
                            val.is_some() {
                            if let Ok(pg) = val.unwrap().parse::<u32>() {
                                page = pg;
                            }
                            break;
                        }
                    }
                    resource::get_index_page(&self.cached_articles, page)
                } else {
                    resource::get_index_page(&self.cached_articles, 0)
                }
            )
        } else {
            let local_path = path_buf![&CONFIGS.root_dir, &path];
            // Materials. Read only known file formats.
            resource_to_response(
                &path,
                if let Some(media_type) =
                    resource::deduce_type_by_ext(&local_path) {
                    resource::get_material(&local_path, media_type)
                } else {
                    resource::get_article(&local_path)
                }
            )
        }
    }
    /// Response to incoming requests.
    fn make_response(&self, req: &Request) -> Response {
        /// Map search directory to local storage directory.
        fn map_search_dir(search_dir: &str) -> Option<&str> {
            match search_dir {
                "post" => Some(&CONFIGS.post_dir),
                "static" => Some(&CONFIGS.static_dir),
                // "error", "template" => No, these are not directly exposed.
                _ => None,
            }
        }

        info!("Request for {} from {}.", req.url, req.remote_addr);
        
        // Only GET method is allowed.
        if req.method != Method::Get {
            warn!("Invalid HTTP method.");
            return gen_error(status::MethodNotAllowed);
        }
        // $path is guaranteed to have at least 1 element.
        let path = req.url.path();
        // Assign different search directory for different root. If the requested
        // thing doesn't exist, ignore with 404 returned.
        let search_dir = path.get(0).unwrap().to_owned();
        // Read data from storage.
        match map_search_dir(&search_dir) {
            Some(dir) => {
                self.make_response_for_dir(
                    dir.to_owned(),
                    path[1..].join("/"),
                    search_dir == "post"
                )
            },
            None => {
                self.make_response_for_root(
                    path.join("/"),
                    req.url.query()
                )
            },
        }
    }

    fn response(&self, req: &mut Request) -> IronResult<Response> {
        let res = Ok(self.make_response(&req));
        return res;
    }
}

/// Writium controller.
pub struct Writium {
    shared: Arc<RwLock<WritiumServer>>,
    listening: iron::Listening,
    ssl_listening: Option<iron::Listening>,
}
impl Writium {
    pub fn new() -> Writium {
        // Use Rwlock to ensure there is no read / write conflicts
        let shared = Arc::new(RwLock::new(WritiumServer {
            cached_articles: resource::gen_cache(),
        }));
        let shared_remote = shared.clone();
        let handler = move |req: &mut Request| {
            if let Ok(locked) = shared_remote.read() {
                (*locked).response(req)
            } else {
                error!("Unable to read-lock.");
                Ok(iron::Response::with((iron::status::InternalServerError)))
            }
        };
        // If `ssl_identity_path` is empty, there is no identity provided.
        // So SSL is disabled, run only HTTP server.
        if CONFIGS.ssl_identity_path.is_empty() {
            Writium {
                ssl_listening: None,
                listening: Iron::new(handler)
                    .http(&CONFIGS.host_addr)
                    .unwrap(),
                shared: shared,
            }
        }
        // If identity is reachable, run SSL server to respond to all request
        // while all HTTP requests are 301ed to HTTPS server.
        else {
            let ssl = NativeTlsServer::new(
                &CONFIGS.ssl_identity_path,
                &CONFIGS.ssl_password
            ).unwrap();

            Writium {
                ssl_listening: Some(
                    Iron::new(handler)
                        .https(&CONFIGS.host_addr_secure, ssl)
                        .unwrap()
                ),
                listening: Iron::new(move |req: &mut Request|{
                    info!("Upgrading request for {} from {} to HTTPS.",
                        req.url, req.remote_addr);
                    let mut res: Response =
                        iron::Response::with((status::MovedPermanently));
                    let mut url: url::Url = req.url.clone().into();
                    let _ = url.set_scheme("https");
                    res.headers.set_raw("Location",
                        vec![url.as_str().as_bytes().to_owned() as Vec<u8>]);
                    Ok(res)
                    }).http(&CONFIGS.host_addr)
                        .unwrap(),
                shared: shared,
            }
        }
    }

    fn close(&mut self) {
        let _ = self.listening.close();
        if let Some(ref mut sl) = self.ssl_listening {
            let _ = sl.close();
        }
    }

    fn interpret_command(&mut self, command: &str, args: &[&str]) -> bool {
        match command {
            "close" => {
                self.close();
                return true;
            },
            "remove_cache" => resource::remove_cache(),
            "recache" => {
                if let Ok(mut locked) = self.shared.write() {
                    resource::remove_cache();
                    (*locked).cached_articles = resource::gen_cache();
                } else {
                    error!("Unable to write-lock.");
                }
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
impl Drop for Writium {
    fn drop(&mut self) {
        resource::remove_cache();
        self.close();
    }
}
