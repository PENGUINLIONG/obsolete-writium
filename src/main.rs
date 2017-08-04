#[macro_use]
extern crate log;
extern crate env_logger;

mod writus;

fn main() {
    let _ = env_logger::init();
    writus::start();
}
