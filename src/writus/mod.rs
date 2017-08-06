extern crate chrono;
extern crate iron;
extern crate json;
extern crate getopts;
extern crate markdown;

use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use self::iron::prelude::*;
use self::iron::method::Method;
use self::iron::status;

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
            AddSlash =>
                gen_redirection(&(format!("/{}/", &path))),
        },
        None => gen_error_page(status::NotFound),
    }
}

struct SharedData {
    cached_articles: resource::CachedAriticles,
}
impl SharedData {
    fn gen_digest(&self, page: u32) -> String {
        let base = page * &CONFIGS.digests_per_page;
        let requested_articles = self.cached_articles.iter();

        let mut template_path = PathBuf::new();
        template_path.push(&CONFIGS.template_dir);
        template_path.push(&CONFIGS.digest_template_path);
        let template = resource::load_text_resource(template_path.as_path())
            .unwrap_or_default();

        let mut vars = template::TemplateVariables::new();

        let mut digest_collected = String::new();

        for (_, article_name) in requested_articles
            .skip(base as usize)
            .take(CONFIGS.digests_per_page as usize) {
            let mut article_path = PathBuf::new();
            article_path.push(&CONFIGS.post_dir);
            article_path.push(&article_name);
            article_path.push("content.md");
            match resource::load_text_resource(article_path.as_path()) {
                Some(content) => {
                    let digest_parts: Vec<&str> = content.lines()
                        .filter(|s: &&str| !s.trim_left().is_empty())
                        .take(2)
                        .collect();
                    let digest_part = markdown::to_html(&(digest_parts.join("\r\n\r\n")));
                    vars.insert("name".to_owned(), format!("/post/{}/", &article_name));
                    vars.insert("digest".to_owned(), digest_part);
                    digest_collected += &vars.fill_template(&template)
                        .unwrap_or_default();
                },
                None => {},
            }
        }
        digest_collected
    }

    fn make_response_for_dir(&self, local_dir: String, path: String, in_post_dir: bool) -> Response {
        let mut local_path = PathBuf::new();
        local_path.push(&local_dir);
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
        resource_to_response(
            &path,
            resource::get_resource(local_path.as_path(), in_post_dir)
        )
    }
    fn make_response_for_root(&self, local_dir: String, path: String, query: Option<&str>)
        -> Response {
        let mut local_path = PathBuf::new();
        local_path.push(&local_dir);
        if path.is_empty() {
            // Index page.
            info!("Request for index.");
            local_path.push("index");

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
                    resource::get_index_page(self.gen_digest(page), page)
                } else {
                    resource::get_index_page(self.gen_digest(0), 0)
                }
            )
        } else {
            // Materials. Read only known file formats.
            local_path.push(&path);
            resource_to_response(
                &path,
                if let Some(media_type) =
                    resource::deduce_type_by_ext(local_path.as_path()) {
                    resource::get_material(local_path.as_path(), media_type)
                } else { None }
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
        // Read data from storage.
        match map_search_dir(&search_dir) {
            Some(dir) => {
                info!("Search directory is {}.", search_dir);
                self.make_response_for_dir(dir.to_owned(), path[1..].join("/"), search_dir == "post")
            },
            None => {
                info!("Search directory is root.", );
                self.make_response_for_root(CONFIGS.root_dir.to_owned(), path.join("/"),
                    req.url.query())
            },
        }
    }

    fn response(&self, req: &mut Request) -> IronResult<Response> {
        info!("-- Response Begin --");
        let res = Ok(self.make_response(&req));
        info!("-- Response End --");
        return res;
    }
}

pub struct Writus {
    shared: Arc<RwLock<SharedData>>,
    listening: iron::Listening,
}
impl Writus {
    pub fn new() -> Writus {
        // Use Rwlock to ensure there is no read / write conflicts
        let shared = Arc::new(RwLock::new(SharedData {
            cached_articles: resource::gen_cache(),
        }));
        let shared_remote = shared.clone();
        Writus {
            listening: Iron::new(move |req: &mut Request| {
                if let Ok(locked) = shared_remote.read() {
                    (*locked).response(req)
                } else {
                    error!("Unable to read-lock.");
                    Ok(iron::Response::with((iron::status::InternalServerError)))
                }
            }).http(&CONFIGS.host_addr).unwrap(),
            shared: shared,
        }
    }

    fn close(&mut self) {
        let _ = self.listening.close();
    }

    fn interpret_command(&mut self, command: &str, args: &[&str]) -> bool {
        match command {
            "close" => {
                self.close();
                return true;
            },
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
impl Drop for Writus {
    fn drop(&mut self) {
        resource::remove_cache();
        self.close();
    }
}
