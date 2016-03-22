#![feature(plugin)]
#![plugin(rocket_macros)]

extern crate rocket;

use rocket::Rocket;
use std::fs::File;

#[route(GET, path = "/")]
fn index() -> File {
    File::open("static/index.html").unwrap()
}

#[route(GET, path = "/<file>")]
fn files(file: &str) -> File {
    File::open(format!("static/{}", file)).unwrap()
}

fn main() {
    Rocket::new("localhost", 8000).mount_and_launch("/", routes![index, files]);
}
