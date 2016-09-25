#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;

use rocket::Rocket;
use rocket::response::{data, Stream};

use std::io::{self, repeat, Repeat, Read, Take};
use std::fs::File;

type LimitedRepeat = Take<Repeat>;

#[get("/")]
fn root() -> data::Plain<Stream<LimitedRepeat>> {
    data::Plain(Stream::from(repeat('a' as u8).take(25000)))
}

#[get("/big_file")]
fn file() -> io::Result<Stream<File>> {
    // Generate this file using: head -c BYTES /dev/random > big_file.dat
    const FILENAME: &'static str = "big_file.dat";
    File::open(FILENAME).map(|file| Stream::from(file))
}

fn main() {
    Rocket::new("localhost", 8000).mount_and_launch("/", routes![root, file]);
}
