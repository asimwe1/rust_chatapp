#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
use rocket::Rocket;

use std::fs::File;
use std::io::Error as IOError;
use std::path::{Path, PathBuf};

#[get("/")]
fn index() -> File {
    File::open("static/index.html").unwrap()
}

#[get("/<file..>")]
fn files(file: PathBuf) -> Result<File, IOError> {
    File::open(Path::new("static/").join(file))
}

fn main() {
    Rocket::new("localhost", 8000).mount_and_launch("/", routes![index, files]);
}
