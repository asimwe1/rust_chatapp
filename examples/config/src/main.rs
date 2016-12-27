#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;

// This example's illustration is the Rocket.toml file.

#[get("/")]
fn hello() -> &'static str {
    "Hello, world!"
}

fn main() {
    rocket::ignite().mount("/hello", routes![hello]).launch()
}
