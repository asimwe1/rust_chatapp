#![feature(plugin)]
#![plugin(rocket_macros)]

extern crate rocket;
use rocket::Rocket;

#[GET(path = "/hello/<name>/<age>")]
fn hello(name: &str, age: i8) -> String {
    format!("Hello, {} year old named {}!", age, name)
}

fn main() {
    Rocket::new("localhost", 8000).mount_and_launch("/", routes![hello]);
}
