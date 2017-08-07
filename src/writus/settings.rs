use std::env::args;
use std::path::Path;
use std::process::exit;
use std::collections::HashMap;

use writus::json::object::Object;

use writus::getopts::{Matches, Options};

use writus::resource;

pub struct WritusConfigs {
    /// Host server address or domain. Must include port number.
    pub host_addr: String,

    /// The directory where posts located.
    pub post_dir: String,
    /// The directory where error pages located.
    pub error_dir: String,
    /// The directory where template files located.
    pub template_dir: String,
    /// The directory where static resources located.
    pub static_dir: String,
    /// The directory where the root path directly mapped to.
    pub root_dir: String,

    /// The directory where cache is output.
    pub cache_dir: String,

    /// Digest template file path in $TEMPLATE_DIR. MUST NOT have slash as
    /// prefix. [default: digest.html]
    pub digest_template_path: String,
    /// Index template file path in $TEMPLATE_DIR. MUST NOT have slash as
    /// prefix. [default: index.html]
    pub index_template_path: String,
    /// Pagination template file path in $TEMPLATE_DIR. MUST NOT have slash as
    /// prefix. [default: pagination.html]
    pub pagination_template_path: String,
    /// Post template file path in $TEMPLATE_DIR. MUST NOT have slash as prefix.
    /// [default: post.html]
    pub post_template_path: String,

    /// Number of digests shown per page on index page. [default: 5]
    pub digests_per_page: u32,
}
impl WritusConfigs {
    fn new() -> WritusConfigs {
        WritusConfigs {
            host_addr: String::new(),

            post_dir: String::new(),
            error_dir: String::new(),
            template_dir: String::new(),
            static_dir: String::new(),
            root_dir: String::new(),
            
            cache_dir: String::new(),
            
            digest_template_path: String::new(),
            index_template_path: String::new(),
            pagination_template_path: String::new(),
            post_template_path: String::new(),

            digests_per_page: 0,
        }
    }
    pub fn from_args() -> WritusConfigs {
        fn match_args() -> Option<(Options, Matches)> {
            let mut options = Options::new();
            options.optflag("h", "help", "Help information");
            let args: Vec<String> = args().collect();
            
            match options.parse(&args[1..]) {
                Ok(matches) => Some((options, matches)),
                Err(_) => None,
            }
        }

        fn fill_setting(configs: &mut WritusConfigs, object: &Object) {
            #[inline]
            fn must_have(obj: &mut HashMap<&str, String>, name: &str)
                -> String {
                match obj.remove(name) {
                    Some(val) => val,
                    None => {
                        error!("\"{}\" is needed but we don't have it.", name);
                        exit(1);
                    },
                }
            }
            #[inline]
            fn have_or(obj: &mut HashMap<&str, String>, name: &str, def: &str)
                -> String {
                match obj.remove(name) {
                    Some(val) => val,
                    None => {
                        info!("\"{}\" is filled by default: {}", name, def);
                        def.to_owned()
                    },
                }
            }

            let mut obj = HashMap::new();
            for (key, val) in object.iter() {
                obj.insert(key, val.to_string());
            }

            configs.host_addr = must_have(&mut obj, "hostAddr");

            configs.post_dir = must_have(&mut obj, "postDir");
            configs.error_dir = must_have(&mut obj, "errorDir");
            configs.template_dir = must_have(&mut obj, "templateDir");
            configs.static_dir = must_have(&mut obj, "staticDir");
            configs.root_dir = must_have(&mut obj, "rootDir");

            configs.cache_dir = must_have(&mut obj, "cacheDir");

            configs.digest_template_path =
                have_or(&mut obj, "digestTemplatePath", "digest.html");
            configs.index_template_path =
                have_or(&mut obj, "indexTemplatePath", "index.html");
            configs.pagination_template_path =
                have_or(&mut obj, "paginationTemplatePath", "pagination.html");
            configs.post_template_path =
                have_or(&mut obj, "postTemplatePath", "post.html");

            configs.digests_per_page = match obj.get("digestsPerPage")
                .unwrap_or(&"5".to_owned())
                .parse::<u32>() {
                Ok(v) => v,
                Err(_) => 5,
            }
        }

        let mut rv = WritusConfigs::new();
        match match_args() {
            Some((options, matches)) => {
                if matches.opt_present("h") {
                    println!("{}", options
                        .usage(&"Usage: writus CONFIG_FILE [options]"));
                    exit(0);
                }
                if matches.free.is_empty() {
                    println!("No configuration file given.");
                    exit(1);
                }
                let path = &matches.free[0];
                match resource::load_json_object(Path::new(&path)) {
                    Some(obj) => fill_setting(&mut rv, &obj),
                    None => {
                        error!("Unable to read configuration file.");
                        exit(1);
                    },
                }
            }
            None => {
                error!("Unable to parse arguments.");
                exit(1);
            }
        }
        rv
    }
}

lazy_static! {
    pub static ref CONFIGS: WritusConfigs = WritusConfigs::from_args();
}
