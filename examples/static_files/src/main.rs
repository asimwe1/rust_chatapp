#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;

use std::io;
use rocket::response::NamedFile;
use std::path::{Path, PathBuf};

#[get("/")]
fn index() -> io::Result<NamedFile> {
    NamedFile::open("static/index.html")
}

#[get("/<file..>")]
fn files(file: PathBuf) -> io::Result<NamedFile> {
    NamedFile::open(Path::new("static/").join(file))
}

fn main() {
    rocket::ignite().mount_and_launch("/", routes![index, files]);
}
