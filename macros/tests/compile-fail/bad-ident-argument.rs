#![feature(plugin)]
#![plugin(rocket_macros)]

extern crate rocket;

#[get("/")]
fn get(_: &str) -> &'static str { "hi" } //~ ERROR argument

fn main() {  }
