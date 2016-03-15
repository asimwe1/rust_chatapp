#![feature(plugin)]
#![plugin(rocket_macros)]

extern crate rocket;
use rocket::Rocket;

#[route(GET, path = "/")]
fn simple() -> &'static str {
    "Hello, simple example world! How is thou?"
}

#[route(GET, path = "/<name>")]
fn hello(name: String) -> String {
    format!("Hello, {}!", name)
}

#[route(PUT, path = "/<x>/<y>")]
fn bye(x: usize, y: usize) -> String {
    format!("{} + {} = {}", x, y, x + y)
}

fn main() {
    let rocket = Rocket::new("localhost", 8000);
    rocket.mount_and_launch("/", routes![simple, hello, bye]);
}
