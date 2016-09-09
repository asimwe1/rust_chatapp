#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
use rocket::Rocket;

#[get("/")]
fn root() -> &'static str {
    "Hello, world!"
}

fn main() {
    Rocket::new("localhost", 8000).mount_and_launch("/", routes![root]);
}
