#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;

use std::io;
use std::path::{Path, PathBuf};

use rocket::response::{NamedFile, Failure};
use rocket::http::StatusCode::NotFound;

#[get("/")]
fn index() -> io::Result<NamedFile> {
    NamedFile::open("static/index.html")
}

#[get("/<file..>")]
fn files(file: PathBuf) -> Result<NamedFile, Failure> {
    NamedFile::open(Path::new("static/").join(file)).map_err(|_| Failure(NotFound))
}

fn main() {
    rocket::ignite().mount("/", routes![index, files]).launch();
}
