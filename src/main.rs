extern crate env_logger;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[macro_use]
extern crate path_buf;

mod writium;

use writium::Writium;

fn main() {
    let _ = env_logger::init();
    let mut instance = Writium::new();
    instance.process_commands();
}
