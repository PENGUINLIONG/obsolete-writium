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

use writus::markdown;

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

pub type CachedArticles = BTreeMap<DateTime<Utc>, String>;

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

/// Ensure the existance of a directory. If needed, any intermediate directory
/// will be created as well.
pub fn ensure_dir(dir_path: &Path) {
    if !dir_path.exists() {
        info!("Directory {} does not exist. Creating one.", dir_path.to_string_lossy());
        if let Err(_) = fs::create_dir_all(dir_path) {
            warn!("Unable to create directory, pages will be generated just-in-time.");
        }
    }
}

//
// Article utilities.
//

/// Get template variables from `metadata.json` and filesystem metadata.
fn get_template_vars(local_path: &Path) -> TemplateVariables {
    let mut vars = TemplateVariables::new();
    vars.read_from_metadata(local_path);
    vars.complete_with_default(local_path);
    vars
}

pub fn get_article_title_content_markdown(local_path: &Path)
    -> Option<(String, String)> {
    let path = path_buf![local_path, "content.md"];
    match load_text_resource(path.as_path()) {
        Some(mut s) => {
            let linebreak_pos = match s.find("\r\n") {
                Some(pos) => pos,
                None => return None,
            };
            // Get title.
            let title_line: String = s.drain(..linebreak_pos).collect();
            let mut pos = 0;
            for (idx, ch) in title_line.char_indices() {
                if ch != '#' { pos = idx; break; }
            }
            let title = title_line[pos..].trim();
            // Get content.
            let content = s.trim();
            Some((title.to_owned(), content.to_owned()))
        },
        None => None,
    }
}

/// Generate article with provided template variables.
fn gen_article_given_vars(local_path: &Path, vars: &mut TemplateVariables) -> Option<Resource> {
    use self::Resource::{Article, InvalidArticle};

    let template_path =
        path_buf![&CONFIGS.template_dir, &CONFIGS.post_template_path];
    let template = match load_text_resource(template_path.as_path()) {
        Some(tmpl) => tmpl,
        None => return Some(InvalidArticle),
    };
    let (title, content) =
        match get_article_title_content_markdown(local_path) {
        Some(tp) => tp,
        None => return Some(InvalidArticle),
    };
    vars.insert("content".to_owned(), markdown::to_html(&content));
    vars.insert("title".to_owned(), title);
    let md_opt = vars.fill_template(&template);
    vars.remove("content");
    match md_opt {
        Some(md) => Some(Article{content: md}),
        None => Some(InvalidArticle),
    }
}

//
// Index utilities.
//

/// Generate digests for a certain page.
fn gen_digests(cached: &CachedArticles, page: u32) -> String {
    let template_path =
        path_buf![&CONFIGS.template_dir, &CONFIGS.digest_template_path];
    let template = load_text_resource(&template_path)
        .unwrap_or_default();

    let mut vars = TemplateVariables::new();
    let mut digest_collected = String::new();
    for (_, article_name) in cached.iter()
        // Page number is 1-based, so minus 1.
        .skip(((page - 1) * &CONFIGS.digests_per_page) as usize)
        .take(CONFIGS.digests_per_page as usize) {
        let path = path_buf![&CONFIGS.post_dir, &article_name];
        vars.read_from_metadata(&path);
        vars.complete_with_default(&path);
        let article_path =
                path_buf![&CONFIGS.post_dir, &article_name];
        if let Some((title, mut content)) =
            get_article_title_content_markdown(&article_path) {
            // Show 50 characters.
            let linebreak_pos = match content.find("\r\n") {
                Some(pos) => pos,
                // Only one line with no content. 
                None => continue,
            };
            let content: String = content.drain(..linebreak_pos).collect();
            vars.insert("path".to_owned(), format!("/post/{}/", &article_name));
            vars.insert("title".to_owned(), title);
            vars.insert("content".to_owned(), markdown::to_html(&content));
            digest_collected += &vars.fill_template(&template)
                .unwrap_or_default();
        }
    }
    digest_collected
}
/// Generate pagination for certain page.
fn gen_pagination(cached: &CachedArticles, page: u32) -> Option<String> {
    let pagination_template_path =
        path_buf![&CONFIGS.template_dir, &CONFIGS.pagination_template_path];
    let pagination_template =
        load_text_resource(&pagination_template_path).unwrap_or_default();

    /// Provide the corresponding page number or empty string depending on the
    /// existence of that page.
    let len = cached.len() as u32;
    let per_page = &CONFIGS.digests_per_page;
    let mut max_page = len / per_page;
    if len % per_page > 0 {
        max_page += 1;
    }

    // It won't underflow as we have hav added 1 to it before.
    let mut page_cur = page - 1;
    let mut vars = TemplateVariables::new();
    if page_cur > 0 && page_cur <= max_page {
        vars.insert("previousPage".to_owned(), page_cur.to_string());
        vars.insert("previousPageLink".to_owned(),
            format!("/?page={}", page_cur).to_string());
    }
    page_cur += 1;
    if page_cur > 0 && page_cur <= max_page {
        vars.insert("thisPage".to_owned(), page_cur.to_string());
    }
    page_cur += 1;
    if page_cur > 0 && page_cur <= max_page
     {
        vars.insert("nextPage".to_owned(), page_cur.to_string());
        vars.insert("nextPageLink".to_owned(),
            format!("/?page={}", page_cur).to_string());
    }
    vars.fill_template(&pagination_template)
}
/// Generate given page of index with given digest. 
fn gen_index_page_given_digest(cached: &CachedArticles, digests: String, page: u32)
    -> Option<String> {
    let index_template_path =
        path_buf![&CONFIGS.template_dir, &CONFIGS.index_template_path];
    let index_template = load_text_resource(&index_template_path)
        .unwrap_or_default();

    let mut vars = TemplateVariables::new();
    vars.insert("digests".to_owned(), digests);
    vars.insert("pagination".to_owned(),
        gen_pagination(cached, page).unwrap_or_default());
    if let Some(filled) = vars.fill_template(&index_template) {
        Some(filled)
    } else {
        None
    }
}

//
// Cache generation.
//

/// Generate cache for articles.
fn gen_article_cache() -> CachedArticles {
    fn gen_single_cache(entry: &Path) -> Option<(DateTime<Utc>, String)> {
        fn parse_date_time(vars: &TemplateVariables, key: &str) -> Option<DateTime<Utc>> {
            if let Some(pd) = vars.get(key) {
                if let Ok(parsed) =
                    DateTime::parse_from_rfc3339(&pd.to_string()) {
                    return Some(DateTime::from_utc(parsed.naive_utc(), Utc));
                }
            }
            None
        }
        // Get metadata of each directory.
        if !entry.is_dir() { return None; }

        let file_name = match entry.file_name().and_then(OsStr::to_str) {
            Some(fname) => fname.to_owned(),
            None => return None,
        };
        let mut cache_path =
            path_buf![&CONFIGS.cache_dir, "post", &file_name];
        cache_path.set_extension("writuscache");

        let article_path = path_buf![entry, ""];
        let mut vars = get_template_vars(&article_path);
        let filled = match gen_article_given_vars(&article_path, &mut vars) {
            Some(Resource::Article { content }) => content,
            _ => return None,
        };
        // In case there is a dot in the file name. set_extension() is
        // not used.
        match File::create(&cache_path) {
            Ok(mut file) => {
                match file.write(filled.as_bytes()) {
                    Ok(_) => {
                        info!("Generated cache: {}", &file_name);
                        match parse_date_time(&vars, "published") {
                            Some(dt) => return Some((dt, file_name)),
                            None => warn!("...But failed to index it."),
                        }
                    },
                    Err(_) => warn!("Failed to generate cache: {}", &file_name),
                }
            },
            Err(_) => warn!("Unable to create cache file: {}", &file_name),
        };
        None
    }

    info!("Generating cache for articles.");
    ensure_dir(Path::new(&CONFIGS.cache_dir));
    // Generate cache for posts.
    ensure_dir(&path_buf![&CONFIGS.cache_dir, "post"]);
    let mut map: CachedArticles = BTreeMap::new();
    match fs::read_dir(&CONFIGS.post_dir) {
        Ok(entries) => for entry in entries {
            if let Ok(en) = entry {
                if let Some((dt, name)) = gen_single_cache(&en.path()) {
                    map.insert(dt, name);
                }
            }
        },
        _ => warn!("Unable to read from post directory."),
    }
    map
}
/// Generate cache for the given page of index.
fn gen_index_page_cache(cached: &CachedArticles, page: u32) {
    info!("Generating cache for index pages.");    
    let file_name = format!("index_{}", page);
    let mut cache_path =
        path_buf![&CONFIGS.cache_dir, &file_name];
    cache_path.set_extension("writuscache");

    let filled =
        match gen_index_page_given_digest(cached, gen_digests(cached, page), page) {
        Some(filed) => filed,
        None => return,
    };
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

/// Generate cache for all articles and the fist page of index.
pub fn gen_cache() -> CachedArticles {
    let cached = gen_article_cache();
    gen_index_page_cache(&cached, 1);
    cached
}

//// Cache loading.
//

fn load_cached_article(local_path: &Path) -> Option<String> {
    match local_path.canonicalize() {
        Ok(name) => {
            let name = match name.file_name().and_then(OsStr::to_str) {
                Some(nm) => nm,
                None => return None,
            };
            let mut cache_path =
                path_buf![&CONFIGS.cache_dir, "post", name];
            cache_path.set_extension("writuscache");
            load_text_resource(&cache_path)
        },
        Err(_) => None,
    }
}
fn load_cached_index_page(page: u32) -> Option<String> {
    let cache_path =
        path_buf![&CONFIGS.cache_dir, format!("index_{}.writuscache", page)];
    load_text_resource(&cache_path)
}

//
// Cache removal.
//

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
    let mut vars = get_template_vars(local_path);
    gen_article_given_vars(local_path, &mut vars)
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
            }
            warn!("Cache not found. Generate page now.");

            // Cache not found, generate now.
            get_article(&local_path)
        } else {
            // Unrecognized resource type.
            None
        },
    }
}

/// Get index page.
pub fn get_index_page(cached: &CachedArticles, page: u32) -> Option<Resource> {
    let real_page = if page == 0 { 1 } else { page };
    
    if let Some(cached) = load_cached_index_page(real_page) {
        info!("Found cache. Use cached page instead.");
        return Some(Resource::Article{ content: cached });
    }
    warn!("Cache not found. Generate page now.");
    let digests = gen_digests(cached, real_page);
    match gen_index_page_given_digest(cached, digests, real_page) {
        Some(content) => Some(Resource::Article{ content: content }),
        None => None,
    }
}
