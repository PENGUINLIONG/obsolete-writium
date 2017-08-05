extern crate env_logger;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

mod writus;

use writus::Writus;

fn main() {
    let _ = env_logger::init();
    let mut instance = Writus::new();
    instance.process_commands();
}
