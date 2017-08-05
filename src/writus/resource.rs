use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use writus::json;
use writus::json::JsonValue;
use writus::json::object::Object;

use writus::settings::CONFIGS;
use writus::template::TemplateVariables;

pub enum Resource {
    Material {
        media_type: String,
        data: Vec<u8>,
    },
    InvalidMaterial,
    Article {
        content: String
    },
    InvalidArticle,
    AddSlash,
}

//
// File input utilities.
//

/// Load resource from local storage.
///
/// Some(~) will be returned if the requested resource is successfully read.
/// None, otherwise.
pub fn load_resource(local_path: &Path) -> Option<Vec<u8>> {
    info!("Looking for file in local storage: {:?}", local_path);
    if let Ok(mut file) = File::open(&local_path) {
        // Fetch content.
        let mut content = Vec::<u8>::new();
        match file.read_to_end(&mut content) {
            Ok(_) => Some(content),
            Err(_) => None,
        }
    } else {
        None
    }
}
pub fn load_text_resource(local_path: &Path) -> Option<String> {
    info!("Looking for text file in local storage: {:?}", local_path);
    if let Ok(mut file) = File::open(&local_path) {
        // Fetch content.
        let mut content = String::new();
        match file.read_to_string(&mut content) {
            Ok(_) => Some(content),
            Err(_) => None,
        }
    } else {
        None
    }
}
pub fn load_json_object(local_path: &Path) -> Option<Object> {
    match load_text_resource(local_path) {
        Some(s) => match json::parse(&s) {
            Ok(JsonValue::Object(obj)) => Some(obj),
            _ => None,
        },
        None => None,
    }
}

//
// Cache management.
//

pub fn gen_cache() {
    fn gen_article_cache(entry: PathBuf) {
        // Get metadata of each directory.
        if !entry.is_dir() { return }

        let mut cache_path = PathBuf::new();
        let file_name = match entry.file_name()
            .and_then(OsStr::to_str) {
            Some(fname) => fname.to_owned(),
            None => return,
        };
        cache_path.push(&CONFIGS.cache_dir);
        cache_path.push(file_name.clone() + &".writuscache");

        let mut article_path = entry;
        article_path.push("");
        let filled = match get_article(article_path.as_path()) {
            Some(Resource::Article { content }) => content,
            _ => return,
        };
        // In case there is a dot in the file name. set_extension() is
        // not used.
        match File::create(&cache_path) {
            Ok(mut file) => {
                match file.write(filled.as_bytes()) {
                    Ok(_) => info!("Generated cache: {}", &file_name),
                    Err(_) => warn!("Failed to generate cache: {}", &file_name),
                }
            },
            Err(_) => warn!("Unable to create cache file: {}", &file_name),
        };
    }

    info!("Generating cache.");
    if !Path::new(&CONFIGS.cache_dir).exists() {
        info!("Cache directory does not exist. Creating one.");
        if let Err(_) = fs::create_dir(&CONFIGS.cache_dir) {
            warn!("Unable to create cache directory, pages will be generated just-in-time.");
        }
    }

    match fs::read_dir(&CONFIGS.post_dir) {
        Ok(entries) => for entry in entries {
            if let Ok(en) = entry {
                gen_article_cache(en.path());
            }
        },
        _ => warn!("Unable to read from post directory."),
    }
}
pub fn remove_cache() {
    info!("Removing all cache.");
    let path = Path::new(&CONFIGS.cache_dir);
    if let Err(_) = fs::remove_dir_all(path) {
        warn!("Unable to remove cache.");
    }
}

//
// High Level resource access.
//

/// Get file extension in path. None is returned if there isn't one.
fn deduce_type_by_ext(local_path: &Path) -> Option<&str> {
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
    local_path.extension().and_then(OsStr::to_str).and_then(map_ext)
}

pub fn get_material(local_path: &Path, media_type: &str) -> Option<Resource> {
    use self::Resource::{Material, InvalidMaterial};

    match load_resource(local_path) {
        Some(data) => Some(Material {
            media_type: media_type.to_owned(),
            data: data,
        }),
        None => Some(InvalidMaterial),
    }
}
pub fn get_article(local_path: &Path) -> Option<Resource> {
    use self::Resource::{Article, InvalidArticle};

    let vars = match TemplateVariables::read_metadata(local_path) {
        Some(v) => v,
        None => return Some(InvalidArticle),
    };

    let mut template_path = PathBuf::new();
    template_path.push(&CONFIGS.template_dir);
    template_path.push(&CONFIGS.post_template_path);
    let template = match load_text_resource(template_path.as_path()) {
        Some(tmpl) => tmpl,
        None => return Some(InvalidArticle),
    };
    match vars.fill_template(&template) {
        Some(filled) => Some(Article{content: filled}),
        None => Some(InvalidArticle),
    }
}
fn load_cached_article(local_path: &Path) -> Option<String> {
    match local_path.canonicalize() {
        Ok(name) => {
            let name = match name.file_name().and_then(OsStr::to_str) {
                Some(nm) => nm,
                None => return None,
            };
            let mut cache_path = PathBuf::new();
            cache_path.push(&CONFIGS.cache_dir);
            cache_path.push(name);
            cache_path.set_extension("writuscache");
            load_text_resource(cache_path.as_path())
        },
        Err(_) => None,
    }
}
/// Get resource file.
pub fn get_resource(local_path: &Path, in_post: bool) -> Option<Resource> {
    use self::Resource::{AddSlash, Article};

    match deduce_type_by_ext(local_path) {
        // Extension present, return material.
        Some(media_type) => get_material(&local_path, media_type),
        // Extension absent, return article.
        None => if in_post { // Article can only be in `./post`.
            // Ensure requested url is in form of `/foo/` rather than `/foo`. It allows
            // the client to acquire resources in the same directory.
            let path_literal = local_path.to_str().unwrap_or_default();
            if !path_literal.ends_with('/') && !path_literal.ends_with('\\') {
                return Some(AddSlash);
            }
            // Look for cached pages first.
            if let Some(cached) = load_cached_article(&local_path) {
                info!("Found cache. Use cached page instead.");
                return Some(Article{ content: cached });
            } else {
                warn!("Cache not found. Generate page now.");
            }

            // Cache not found, generate now.
            get_article(&local_path)
        } else {
            // Unrecognized resource type.
            None
        },
    }
}
