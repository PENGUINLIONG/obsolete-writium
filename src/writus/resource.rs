extern crate chrono;
extern crate json;
extern crate markdown;

use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use self::chrono::Local;

use self::json::JsonValue::Object;

use writus::settings;
use writus::template;

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
fn load_text_resource(local_path: &str) -> Option<String> {
    match load_resource(&local_path) {
        Some(data) => match String::from_utf8(data) {
            Ok(text) => Some(text),
            Err(_) => None,
        },
        None => None,
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

/// Complete template variable map with default value.
fn complete_with_default(local_path: &str, vars: &mut template::TemplateVariables) {
    if !vars.contains_key("author") {
        vars.insert("author".to_owned(), "Akari".to_owned());
    }
    if !vars.contains_key("title") {
        vars.insert("title".to_owned(), "Untitled".to_owned());
    }
    if !vars.contains_key("pub-date") {
        vars.insert("pub-date".to_owned(),
            match fs::metadata(local_path.to_owned() + "content.md") {
            Ok(file_meta) => match file_meta.created() {
                Ok(sys_time) => chrono::DateTime::<Local>::from(sys_time)
                    .format("%Y-%m-%d").to_string(),
                Err(_) => "".to_owned(),
            },
            Err(_) => "".to_owned(),
        });
    }
}
/// Get metadata of given post, set default value if necessary.
fn get_metadata(local_path: &str) -> Option<template::TemplateVariables> {
    let mut vars = template::TemplateVariables::new();

    let content_path = local_path.to_owned() + "content.md";
    let content = match load_text_resource(&content_path) {
        Some(cont) => markdown::to_html(&cont),
        None => return None,
    };
    let metadata_path = local_path.to_owned() + "metadata.json";
    let metadata = match load_text_resource(&metadata_path) {
        Some(cont) => match json::parse(&cont) {
            Ok(Object(parsed)) => parsed,
            Ok(_) => return None,
            Err(_) => return None,
        },
        None => return None,
    };
    for (key, val) in metadata.iter() {
        vars.insert(key.to_owned(), val.as_str().unwrap().to_owned());
    }
    vars.entry("content".to_owned()).or_insert(content);
    complete_with_default(local_path, &mut vars);
    Some(vars)
}

/// Get resource file.
pub fn get_resource(local_path: &str, in_post: bool) -> Option<Resource> {
    use self::Resource::{Article, InvalidArticle, Material, InvalidMaterial, AddSlash};
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
            // Ensure requested url is in form of `/foo/` rather than `/foo`. It allows
            // the client to acquire resources in the same directory.
            if !local_path.ends_with("/") {
                return Some(AddSlash);
            }

            let vars = match get_metadata(local_path) {
                Some(v) => v,
                None => return Some(InvalidArticle),
            };

            let template_path = settings::TEMPLATE_DIR.to_owned() +
                settings::POST_TEMPLATE_PATH;
            let template = match load_text_resource(&template_path) {
                Some(tmpl) => tmpl,
                None => return Some(InvalidArticle),
            };
            match template::fill_template(&template, &vars) {
                Some(filled) => Some(Article{content: filled}),
                None => Some(InvalidArticle),
            }
        } else {
            // Unrecognized resource type.
            None
        }
    }
}
