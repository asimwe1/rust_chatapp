#![feature(plugin)]
#![plugin(rocket_macros)]

extern crate rocket;
use rocket::Rocket;

#[get("/hello/<name>/<age>")]
fn hello(name: &str, age: i8) -> String {
    format!("Hello, {} year old named {}!", age, name)
}

#[get("/hello/<name>")]
fn hi<'r>(name: &'r str) -> &'r str {
    name
}

fn main() {
    Rocket::new("localhost", 8000).mount_and_launch("/", routes![hello, hi]);
}
