use std::env::args;
use std::path::Path;
use std::process::exit;

use writus::getopts::{Matches, Options};

use writus::resource;

pub struct WritusConfigs {
    /// Host address. Must include port number.
    pub host_addr: String,

    /// The directory where posts located.
    pub post_dir: String,
    /// The directory where error pages located.
    pub error_dir: String,
    /// The directory where template files located.
    pub template_dir: String,
    /// The directory where static resources located.
    pub static_dir: String,

    /// The directory where cache is output.
    pub cache_dir: String,

    /// Post template file path in $TEMPLATE_DIR. MUST NOT have slash as prefix.
    pub post_template_path: String,
}
impl WritusConfigs {
    fn new() -> WritusConfigs {
        WritusConfigs {
            host_addr: String::new(),

            post_dir: String::new(),
            error_dir: String::new(),
            template_dir: String::new(),
            static_dir: String::new(),
            
            cache_dir: String::new(),
            
            post_template_path: String::new(),
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
        fn fill_setting(configs: &mut WritusConfigs, key: &str, val: &str)
            -> bool {
            let val = val.to_owned();
            match key {
                "hostAddr" => configs.host_addr = val,

                "postDir" => configs.post_dir = val,
                "errorDir" => configs.error_dir = val,
                "templateDir" => configs.template_dir = val,
                "staticDir" => configs.static_dir = val,
                
                "cacheDir" => configs.cache_dir = val,
                
                "postTemplatePath" => configs.post_template_path = val,
                _ => return false,
            }
            true
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
                    Some(obj) => {
                        let mut count = 0;
                        for (key, val) in obj.iter() {
                            if fill_setting(&mut rv, &key, &val.to_string()) { count += 1 }
                        }
                        if count < 7 {
                            error!("Configuration file is not complete.");
                            exit(1);
                        }
                    },
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
