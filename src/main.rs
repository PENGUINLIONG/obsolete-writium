extern crate chrono;
extern crate env_logger;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[macro_use]
extern crate path_buf;

use std::env;

use env_logger::LogBuilder;
use log::{LogLevelFilter, LogRecord};

mod writium;

use writium::Writium;

fn main() {
    let format = |record: &LogRecord| {
        format!("{} [{}] {}", chrono::Utc::now().to_rfc3339(), record.level(), record.args())
    };
    let mut builder = LogBuilder::new();
    builder.format(format).filter(None, LogLevelFilter::Info);
    if env::var("RUST_LOG").is_ok() {
       builder.parse(&env::var("RUST_LOG").unwrap());
    }
    builder.init();

    let mut instance = Writium::new();
    instance.process_commands();
}
