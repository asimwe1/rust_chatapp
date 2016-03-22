#![feature(plugin)]
#![plugin(rocket_macros)]

extern crate rocket;
use rocket::Rocket;
use std::fs::File;

#[route(GET, path = "/")]
fn root() -> File {
    File::open("/tmp/index.html").unwrap()
}

#[route(GET, path = "/hello/<name>/<age>")]
fn hello(name: &str, age: i8) -> String {
    format!("Hello, {} year old named {}!", age, name)
}

fn main() {
    let rocket = Rocket::new("localhost", 8000);
    rocket.mount_and_launch("/", routes![root, hello]);
}
