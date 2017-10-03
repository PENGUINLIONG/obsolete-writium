use std::collections::BTreeMap;
use std::fs::metadata;
use std::fs::Metadata;
use std::io;
use std::path::Path;
use std::time::SystemTime;

use super::super::chrono;
use chrono::Utc;

use writium::resource;
use writium::settings::CONFIGS;

pub struct TemplateVariables {
    vars: BTreeMap<String, String>,
}

impl TemplateVariables {
    pub fn new() -> TemplateVariables {
        TemplateVariables {
            vars: BTreeMap::new(),
        }
    }
    pub fn read_from_metadata(&mut self, local_path: &Path) {
        let metadata_path = path_buf![local_path, "metadata.json"];
        let metadata = match resource::load_json_object(metadata_path.as_path()) {
            Some(j) => j,
            None => return,
        };
        let map = match metadata.as_object() {
            Some(map) => map,
            _ => return,
        };
        for (key, val) in map {
            self.insert(key.to_owned(), val.as_str().unwrap().to_owned());
        }
    }

    fn get_fragment(&self, rel_path: &Path) -> Option<String> {
        let path = path_buf![&CONFIGS.template_dir, rel_path];
        resource::load_text_resource(path.as_path())
            .and_then(|s| self.fill_template(&s))
    }

    pub fn fill_template(&self, template: &str) -> Option<String> {
        debug!("Filling template.");
        let mut template = template.to_owned();
        let mut rv = String::new();
        loop {
            match template.find("<?") {
                Some(beg) => rv.extend(template.drain(..beg)),
                // No more processing instructions, get out of the loop.
                None => return Some(rv + &template),
            }
            match template.find("?>") {
                Some(end) => {
                    if end < 2 {
                        // Tag beginning and ending overlaps.
                        return None;
                    }
                    {
                        let parts: Vec<&str> = template[2..end].splitn(2, ' ')
                            .collect();
                        if parts[0] == "frag" {
                            // Insert fragment.
                            let frag_path = parts[1].trim();
                            debug!("Inline fragment: {}", frag_path);
                            rv += &self.get_fragment(Path::new(frag_path))
                                .unwrap_or_default();
                        } else if parts[0] == "var" {
                            // Insert variable.
                            let var_name = parts[1].trim();
                            debug!("Insert variable: {}", var_name);
                            if let Some(st) = self.get(var_name) {
                                rv += st;
                            }
                        }
                    }
                    // Ignore unknown processing instructions.
                    template.drain(..(end + 2));
                },
                None => return None,
            }
        }
    }

    /// Complete template variable map with default value.
    pub fn complete_with_default(&mut self, local_path: &Path) {
        fn get_meta_dt<F: FnOnce(&Metadata) -> io::Result<SystemTime>>(local_path: &Path, dt_fn: F)
            -> Option<String> {
            let path = path_buf![local_path, "content.md"];
            match metadata(path) {
                Ok(file_meta) => match (dt_fn)(&file_meta) {
                    Ok(sys_time) => Some(chrono::DateTime::<Utc>::from(sys_time)
                        .to_rfc3339()),
                    Err(_) => None,
                },
                Err(_) => None,
            }
        }

        // Can be overridden by user.

        if !self.contains_key("author") {
            self.insert("author".to_owned(), "Akari".to_owned());
        }

        // Now `title` is the first line of an article.

        if !self.contains_key("published") {
            self.insert("published".to_owned(),
                get_meta_dt(local_path, Metadata::created)
                    .unwrap_or_default()
            );
        }

        // Cannot be overridden by user.

        // Variable `content` will be added when article is going to be
        // generated. After generation, it will immediately be removed because
        // it takes too much memory.
        self.insert("created".to_owned(),
            get_meta_dt(local_path, Metadata::created)
                .unwrap_or_default()
        );
        self.insert("modified".to_owned(),
            get_meta_dt(local_path, Metadata::modified)
                .unwrap_or_default()
        );
    }

    #[inline]
    pub fn contains_key(&self, key: &str) -> bool {
        self.vars.contains_key(key)
    }
    #[inline]
    pub fn insert(&mut self, key: String, value: String) -> Option<String> {
        self.vars.insert(key, value)
    }
    #[inline]
    pub fn get(&self, key: &str) -> Option<&String> {
        self.vars.get(key)
    }
    #[inline]
    pub fn remove(&mut self, key: &str) -> Option<String> {
        self.vars.remove(key)
    }
}
