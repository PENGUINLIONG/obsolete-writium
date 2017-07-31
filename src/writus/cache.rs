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
        println!("Generating cache.");
        if !Path::new(settings::CACHE_DIR).exists() {
            println!("Cache directory does not exist. Creating one.");
            if let Err(_) = fs::create_dir(settings::CACHE_DIR) {
                println!("Unable to create cache directory, pages will be generated just-in-time.");
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
                            Ok(_) => println!("Generated cache for: {}", file_name),
                            Err(_) => println!("Failed to write to cache file."),
                        }
                    },
                    Err(_) => { println!("Unable to create cache file for {}", file_name); },
                };
            }
        } else {
            println!("Unable to read from post directory.");
        }
    }
}
impl Drop for Cache {
    fn drop(&mut self) {
        println!("Removing all cache.");
        let path = Path::new(settings::CACHE_DIR);
        if !path.exists() {
            println!("Cache directory does not exist.");
            return;
        }
        if let Err(_) = fs::remove_dir_all(path) {
            println!("Unable to remove cache.");
        }
    }
}
