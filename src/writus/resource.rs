use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use writus::chrono::{DateTime, Utc};

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

pub type CachedAriticles = BTreeMap<DateTime<Utc>, String>;

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

pub fn gen_cache() -> CachedAriticles {
    fn gen_article_cache(entry: &Path) {
        // Get metadata of each directory.
        if !entry.is_dir() { return }

        let file_name = match entry.file_name()
            .and_then(OsStr::to_str) {
            Some(fname) => fname.to_owned(),
            None => return,
        };
        let cache_path =
            path_buf![&CONFIGS.cache_dir, file_name.clone() + &".writuscache"];

        let article_path = path_buf![entry, ""];
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
    fn get_pubdate(entry: &Path) -> Option<DateTime<Utc>> {
        let metadata_path = path_buf![entry, "metadata.json"];
        if let Some(obj) = load_json_object(metadata_path.as_path()) {
            if let Some(pd) = obj.get("pubDate") {
                if let Ok(parsed) =
                    DateTime::parse_from_rfc3339(&pd.to_string()) {
                    return Some(DateTime::from_utc(parsed.naive_utc(), Utc));
                }
            }
        }
        // TODO: Loop over and get time and sort. Ignore all non-timed articles.
        let content_path = path_buf![entry, "content.md"];
        match fs::metadata(content_path) {
            Ok(file_meta) => match file_meta.created() {
                Ok(sys_time) =>
                    Some(DateTime::<Utc>::from(sys_time)),
                Err(_) => None,
            },
            Err(_) => None,
        }
    }

    info!("Generating cache.");
    if !Path::new(&CONFIGS.cache_dir).exists() {
        info!("Cache directory does not exist. Creating one.");
        if let Err(_) = fs::create_dir(&CONFIGS.cache_dir) {
            warn!("Unable to create cache directory, pages will be generated just-in-time.");
        }
    }

    let mut map: CachedAriticles = BTreeMap::new();
    match fs::read_dir(&CONFIGS.post_dir) {
        Ok(entries) => for entry in entries {
            if let Ok(en) = entry {
                let path = en.path();
                gen_article_cache(&path);
                let article_pub_date = get_pubdate(&path);
                let article_name = en.file_name().into_string();
                if article_pub_date.is_some() &&
                    article_name.is_ok() {
                    map.insert(article_pub_date.unwrap(),
                        article_name.unwrap());
                }
            }
        },
        _ => warn!("Unable to read from post directory."),
    }
    map
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
pub fn deduce_type_by_ext(local_path: &Path) -> Option<&str> {
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
            "ico" => Some("image/x-icon"),
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

    let mut vars = TemplateVariables::new();
    vars.read_from_metadata(local_path);
    vars.complete_with_default(local_path);

    let template_path =
        path_buf![&CONFIGS.template_dir, &CONFIGS.post_template_path];
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
            let mut cache_path =
                path_buf![&CONFIGS.cache_dir, name];
            cache_path.set_extension("writuscache");
            load_text_resource(&cache_path)
        },
        Err(_) => None,
    }
}
/// Get resource file.
pub fn get_resource(local_path: &Path, can_be_article: bool)
    -> Option<Resource> {
    use self::Resource::{AddSlash, Article};

    match deduce_type_by_ext(local_path) {
        // Extension present, return material.
        Some(media_type) => get_material(&local_path, media_type),
        // Extension absent, return article.
        // Article can only be the index page or in `./post`.
        None => if can_be_article {
            // Ensure requested url is in form of `/foo/` rather than `/foo`. It
            // allows the client to acquire resources in the same directory.
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

pub fn get_index_page(digest: String, page: u32) -> Option<Resource> {
    fn make_pagination(page: u32) -> Option<String> {
        let pagination_template_path =
            path_buf![&CONFIGS.template_dir, &CONFIGS.pagination_template_path];
        let pagination_template =
            load_text_resource(&pagination_template_path).unwrap_or_default();

        let mut vars = TemplateVariables::new();
        vars.insert("page".to_owned(), page.to_string());
        vars.fill_template(&pagination_template)
    }
    let index_template_path =
        path_buf![&CONFIGS.template_dir, &CONFIGS.index_template_path];
    let index_template = load_text_resource(&index_template_path)
        .unwrap_or_default();

    let mut vars = TemplateVariables::new();
    vars.insert("digests".to_owned(), digest);
    vars.insert("pagination".to_owned(),
        make_pagination(page).unwrap_or_default());
    if let Some(filled) = vars.fill_template(&index_template) {
        Some(Resource::Article{ content: filled })
    } else { None }
}
