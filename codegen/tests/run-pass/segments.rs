#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]

extern crate rocket;

use std::path::PathBuf;

#[post("/<a>/<b..>")]
fn get(a: String, b: PathBuf) -> String {
    format!("{}/{}", a, b.to_string_lossy())
}

fn main() {
    let _ = routes![get];
}
