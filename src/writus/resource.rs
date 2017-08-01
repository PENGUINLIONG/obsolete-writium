use std::ffi::OsStr;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use writus::settings;
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

/// Load resource from local storage.
///
/// Some(~) will be returned if the requested resource is successfully read.
/// None, otherwise.
fn load_resource(local_path: &Path) -> Option<Vec<u8>> {
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
fn load_text_resource(local_path: &Path) -> Option<String> {
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
    template_path.push(settings::TEMPLATE_DIR);
    template_path.push(settings::POST_TEMPLATE_PATH);
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
            cache_path.push(settings::CACHE_DIR);
            cache_path.push(name);
            cache_path.set_extension("writuscache");
            load_text_resource(cache_path.as_path())
        },
        Err(_) => None,
    }
}
/// Get resource file.
pub fn get_resource(local_path: &str, in_post: bool) -> Option<Resource> {
    use self::Resource::{AddSlash, Article};

    let path = Path::new(&local_path);
    match deduce_type_by_ext(Path::new(&local_path)) {
        // Extension present, return material.
        Some(media_type) => get_material(&path, media_type),
        // Extension absent, return article.
        None => if in_post { // Article can only be in `./post`.
            // Ensure requested url is in form of `/foo/` rather than `/foo`. It allows
            // the client to acquire resources in the same directory.
            if !local_path.ends_with("/") && !local_path.ends_with("\\") {
                return Some(AddSlash);
            }
            // Look for cached pages first.
            if let Some(cached) = load_cached_article(&path) {
                info!("Found cache. Use cached page instead.");
                return Some(Article{ content: cached });
            } else {
                warn!("Cache not found. Generate page now.");
            }

            // Cache not found, generate now.
            get_article(&path)
        } else {
            // Unrecognized resource type.
            None
        },
    }
}
