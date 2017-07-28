use std::fs::File;
use std::io::Read;
use std::path::Path;

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
}

/// Load resource from local storage.
///
/// Some(~) will be returned if the requested resource is successfully read.
/// None, otherwise.
fn load_resource(local_path: &str) -> Option<Vec<u8>> {
    println!("Looking for file in local storage: {}", local_path);
    let local_path = Path::new(&local_path);
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

/// Get file extension in path. None is returned if there isn't one.
fn deduct_type_by_ext(local_path: &str) -> Option<&str> {
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
    match local_path.rfind(".") {
        Some(ext_pos) => map_ext(&local_path[(ext_pos + 1)..]),
        None => None,
    }
}

/// Get resource file.
pub fn get_resource(local_path: &str, in_post: bool) -> Option<Resource> {
    use self::Resource::{Article, InvalidArticle, Material, InvalidMaterial};
    match deduct_type_by_ext(&local_path) {
        // Extension present, return material.
        Some(media_type) => match load_resource(local_path) {
            Some(data) => Some(Material {
                media_type: media_type.to_owned(),
                data: data,
            }),
            None => Some(InvalidMaterial),
        },
        // Extension absent, return article.
        None => if in_post { // Article can only be in `./post`.
            match load_resource(&(local_path.to_owned() + ".md")) {
                Some(data) => match String::from_utf8(data) {
                    Ok(content) => Some(Article {
                        content: content,
                    }),
                    Err(_) => Some(InvalidArticle),
                },
                None => Some(InvalidArticle),
            }
        } else {
            None
        }
    }
}
