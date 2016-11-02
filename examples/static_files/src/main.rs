#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;

use std::io;
use std::path::{Path, PathBuf};

use rocket::response::NamedFile;

#[get("/")]
fn index() -> io::Result<NamedFile> {
    NamedFile::open("static/index.html")
}

#[get("/<file..>")]
fn files(file: PathBuf) -> io::Result<NamedFile> {
    NamedFile::open(Path::new("static/").join(file))
}

fn main() {
    rocket::ignite().mount("/", routes![index, files]).launch();
}
