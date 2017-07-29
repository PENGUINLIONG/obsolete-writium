extern crate json;

use std::fs::File;
use std::io::Read;
use std::collections::BTreeMap;

use writus::settings;

pub type TemplateVariables = BTreeMap<String, String>;

fn get_fragment(path: &str, vars: &TemplateVariables) -> String {
    match File::open(settings::TEMPLATE_DIR.to_owned() + path) {
        Ok(mut file) => {
            let mut content = String::new();
            match file.read_to_string(&mut content) {
                Ok(_) => match fill_template(&content, &vars) {
                    Some(filled) => filled,
                    None => "".to_owned(),
                },
                Err(_) => "".to_owned(),
            }
        },
        Err(_) => "".to_owned(),
    }
}
fn get_variable(name: &str, vars: &TemplateVariables) -> String {
    match vars.get(&name.to_lowercase()) {
        Some(var) => var.to_owned(), ////////////////////////////////////////////////// check existency after assignment.
        None => "".to_owned(),
    }
}

pub fn fill_template(template: &str, vars: &TemplateVariables) -> Option<String> {
    println!("Filling template.");
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
                    let parts: Vec<&str> = template[2..end].splitn(2, ' ').collect();
                    if parts[0] == "frag" {
                        // Insert fragment.
                        let frag_path = parts[1].trim();
                        println!("Inline fragment: {}", frag_path);
                        rv += &get_fragment(frag_path, vars);
                    } else if parts[0] == "var" {
                        // Insert variable.
                        let var_name = parts[1].trim();
                        println!("Insert variable: {}", var_name);
                        rv += &get_variable(var_name, vars);
                    }
                }
                // Ignore unknown processing instructions.
                template.drain(..(end + 2));
            },
            None => return None,
        }
    }
}
