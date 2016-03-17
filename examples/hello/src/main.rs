#![feature(plugin)]
#![plugin(rocket_macros)]

extern crate rocket;
use rocket::Rocket;
use rocket::response::{HypResponse, HypFresh, Responder};

use std::fs::File;

// #[route(GET, path = "/")]
// fn simple() -> &'static str {
//     "Hello, simple example world! How is thou?"
// }

#[route(GET, path = "/")]
fn simple() -> File {
    File::open("/tmp/index.html").unwrap()
}

#[route(GET, path = "/hello/")]
fn simple2() -> &'static str {
    "Hello, world!"
}

#[route(GET, path = "/hello")]
fn simple3() -> String {
    String::from("Hello, world!")
}

#[route(GET, path = "/<name>/<age>")]
fn simple4(name: &str, age: i8) -> &str {
    name
}

#[route(GET, path = "/something")]
fn simple5() -> &'static str {
    "hi"
}

fn main() {
    let mut rocket = Rocket::new("localhost", 8000);
    rocket.mount("/", routes![simple, simple2, simple3, simple4, simple5]);
    rocket.mount_and_launch("/hello/", routes![simple, simple3, simple4, simple5]);
}
