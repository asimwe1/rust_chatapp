#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;

#[get("/")]
#[allow(unmounted_route)]
pub fn hello() -> &'static str {
    "Hello, world!"
}
