extern crate chrono;
extern crate iron;
extern crate json;
extern crate markdown;

mod cache;
mod resource;
mod response_gen;
mod server;
mod settings;
mod template;

pub fn start() {
    let mut server = server::Server::new();
    server.process_commands();
}
