#![feature(plugin)]
#![plugin(rocket_macros)]

extern crate rocket;
use rocket::Rocket;

#[get("/users/<name>")]
fn user(name: &str) -> Option<&'static str> {
    if name == "Sergio" {
        Some("Hello, Sergio!")
    } else {
        None
    }
}

fn main() {
    Rocket::new("localhost", 8000).mount_and_launch("/", routes![user]);
}
