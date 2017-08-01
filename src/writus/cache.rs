use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use writus::settings;
use writus::resource;

pub struct Cache {}

impl Cache {
    pub fn new() -> Cache {
        Cache{}
    }
    pub fn gen_cache(&self) {
        info!("Generating cache.");
        if !Path::new(settings::CACHE_DIR).exists() {
            info!("Cache directory does not exist. Creating one.");
            if let Err(_) = fs::create_dir(settings::CACHE_DIR) {
                warn!("Unable to create cache directory, pages will be generated just-in-time.");
                return;
            }
        }

        if let Ok(entries) = fs::read_dir(settings::POST_DIR) {
            for entry in entries {
                // Get metadata of each directory.
                let entry = match entry { Ok(en) => en, Err(_) => continue, };
                if !entry.path().is_dir() { continue }

                let mut article_path = entry.path();
                article_path.push("");
                let filled =
                    match resource::get_article(article_path.as_path()) {
                    Some(resource::Resource::Article { content }) => content,
                    _ => continue,
                };

                let mut cache_path = PathBuf::new();
                let file_name = entry.file_name().into_string().unwrap();
                cache_path.push(settings::CACHE_DIR);
                cache_path.push(file_name.clone() + &".writuscache");
                match File::create(cache_path.as_path()) {
                    Ok(mut file) => {
                        match file.write(filled.as_bytes()) {
                            Ok(_) => info!("Generated cache for: {}", file_name),
                            Err(_) => warn!("Failed to write to cache file."),
                        }
                    },
                    Err(_) => { warn!("Unable to create cache file for {}", file_name); },
                };
            }
        } else {
            warn!("Unable to read from post directory.");
        }
    }
}
impl Drop for Cache {
    fn drop(&mut self) {
        info!("Removing all cache.");
        let path = Path::new(settings::CACHE_DIR);
        if let Err(_) = fs::remove_dir_all(path) {
            warn!("Unable to remove cache.");
        }
    }
}
