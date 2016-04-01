#![feature(plugin)]
#![plugin(rocket_macros)]
extern crate rocket;

use rocket::Rocket;
use rocket::response::Redirect;

#[route(GET, path = "/")]
fn root() -> Redirect {
    Redirect::to("/login")
}

#[route(GET, path = "/login")]
fn login() -> &'static str {
    "Hi! Please log in before continuing."
}

fn main() {
    Rocket::new("localhost", 8000).mount_and_launch("/", routes![root, login]);
}
