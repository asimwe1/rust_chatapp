#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;

use rocket::Rocket;
use rocket::response::{Plain, Stream};

use std::io::{repeat, Repeat, Read, Take};

type LimitedRepeat = Take<Repeat>;

#[get("/")]
fn root() -> Plain<Stream<LimitedRepeat>> {
    Plain(Stream::from(repeat('a' as u8).take(25000)))
}

fn main() {
    Rocket::new("localhost", 8000).mount_and_launch("/", routes![root]);
}
