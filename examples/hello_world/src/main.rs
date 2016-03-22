#![feature(plugin)]
#![plugin(rocket_macros)]

extern crate rocket;
use rocket::Rocket;

#[route(GET, path = "/")]
fn root() -> &'static str {
    "Hello, world!"
}

fn main() {
    Rocket::new("localhost", 8000).mount_and_launch("/", routes![root]);
}
