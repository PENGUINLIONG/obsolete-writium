#[macro_use]
extern crate log;
extern crate env_logger;

mod writus;

use writus::Writus;

fn main() {
    let _ = env_logger::init();
    let mut instance = Writus::new();
    instance.process_commands();
}
