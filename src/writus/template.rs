use std::fs;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use writus::chrono;
use writus::chrono::Utc;
use writus::markdown;

use writus::resource;
use writus::settings::CONFIGS;

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
        let mut metadata_path = PathBuf::new();
        metadata_path.push(local_path);
        metadata_path.push("metadata.json");
        let metadata = match resource::load_json_object(metadata_path.as_path()) {
            Some(j) => j,
            None => return,
        };
        for (key, val) in metadata.iter() {
            self.insert(key.to_owned(), val.as_str().unwrap().to_owned());
        }
    }

    fn get_fragment(&self, rel_path: &Path) -> Option<String> {
        let mut path = PathBuf::new();
        path.push(&CONFIGS.template_dir);
        path.push(rel_path);
        resource::load_text_resource(path.as_path())
            .and_then(|s| self.fill_template(&s))
    }
    fn get_variable(&self, name: &str) -> Option<String> {
        match self.vars.get(&name.to_owned()) {
            Some(var) => Some(var.to_owned()),
            None => None,
        }
    }

    pub fn fill_template(&self, template: &str) -> Option<String> {
        info!("Filling template.");
        let mut template = template.to_owned();
        let mut rv = String::new();
        loop {
            match template.find("<?") {
                Some(beg) => rv.extend(template.drain(..beg)),
                // No more processing instructions, get out of the loop.
                None => return Some(rv),
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
                            info!("Inline fragment: {}", frag_path);
                            rv += &self.get_fragment(Path::new(frag_path))
                                .unwrap_or_default();
                        } else if parts[0] == "var" {
                            // Insert variable.
                            let var_name = parts[1].trim();
                            info!("Insert variable: {}", var_name);
                            rv += &self.get_variable(var_name)
                                .unwrap_or_default();
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
        fn get_content(local_path: &Path) -> Option<String> {
            let mut path = PathBuf::new();
            path.push(local_path);
            path.push("content.md");
            match resource::load_text_resource(path.as_path()) {
                Some(s) => Some(markdown::to_html(&s)),
                None => None,
            }
        }
        fn get_create_date(local_path: &Path) -> Option<String> {
            let mut path = PathBuf::new();
            path.push(local_path);
            path.push("content.md");
            match fs::metadata(local_path) {
                Ok(file_meta) => match file_meta.created() {
                    Ok(sys_time) => Some(chrono::DateTime::<Utc>::from(sys_time)
                            .to_rfc3339()),
                    Err(_) => None,
                },
                Err(_) => None,
            }
        }

        if !self.contains_key("author") {
            self.insert("author".to_owned(), "Akari".to_owned());
        }
        if !self.contains_key("title") {
            self.insert("title".to_owned(), "Untitled".to_owned());
        }
        if !self.contains_key("content") {
            self.insert("content".to_owned(), get_content(local_path)
                .unwrap_or_default());
        }
        if !self.contains_key("pub-date") {
            self.insert("pub-date".to_owned(), get_create_date(local_path)
                .unwrap_or_default());
        }
    }

    #[inline]
    pub fn contains_key(&self, key: &str) -> bool {
        self.vars.contains_key(key)
    }
    #[inline]
    pub fn insert(&mut self, key: String, value: String) -> Option<String> {
        self.vars.insert(key, value)
    }
}
