#![feature(plugin)]
#![plugin(rocket_macros)]

extern crate rocket;

use rocket::Rocket;
use std::fs::File;
use std::io::Error as IOError;

#[route(GET, path = "/")]
fn index() -> File {
    File::open("static/index.html").unwrap()
}

#[route(GET, path = "/<file>")]
fn files(file: &str) -> Result<File, IOError> {
    File::open(format!("static/{}", file))
}

fn main() {
    Rocket::new("localhost", 8000).mount_and_launch("/", routes![index, files]);
}
