use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use writus::resource;
use writus::settings;

pub struct CacheGuard;
impl CacheGuard {
    pub fn new() -> CacheGuard {
        fn gen_article_cache(entry: PathBuf) {
            // Get metadata of each directory.
            if !entry.is_dir() { return }

            let mut cache_path = PathBuf::new();
            let file_name = match entry.file_name()
                .and_then(OsStr::to_str) {
                Some(fname) => fname.to_owned(),
                None => return,
            };
            cache_path.push(settings::CACHE_DIR);
            cache_path.push(file_name.clone() + &".writuscache");

            let mut article_path = entry;
            article_path.push("");
            let filled = match resource::get_article(article_path.as_path()) {
                Some(resource::Resource::Article { content }) => content,
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
        if !Path::new(settings::CACHE_DIR).exists() {
            info!("Cache directory does not exist. Creating one.");
            if let Err(_) = fs::create_dir(settings::CACHE_DIR) {
                warn!("Unable to create cache directory, pages will be generated just-in-time.");
                return CacheGuard{};
            }
        }

        match fs::read_dir(settings::POST_DIR) {
            Ok(entries) => for entry in entries {
                if let Ok(en) = entry {
                    gen_article_cache(en.path());
                }
            },
            _ => warn!("Unable to read from post directory."),
        }
        CacheGuard{}
    }
}
impl Drop for CacheGuard {
    fn drop(&mut self) {
        info!("Removing all cache.");
        let path = Path::new(settings::CACHE_DIR);
        if let Err(_) = fs::remove_dir_all(path) {
            warn!("Unable to remove cache.");
        }
    }
}
