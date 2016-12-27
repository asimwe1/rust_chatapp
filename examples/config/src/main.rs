#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate config;

#[cfg(test)]
mod tests;

// This example's illustration is the Rocket.toml file.
fn main() {
    rocket::ignite().mount("/hello", routes![config::hello]).launch()
}
