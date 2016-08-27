#![feature(plugin)]
#![plugin(rocket_macros)]

extern crate rocket;
use rocket::Rocket;

#[GET(path = "/hello/<name>/<age>")]
fn hello(name: &str, age: i8) -> String {
    format!("Hello, {} year old named {}!", age, name)
}

// FIXME: Add 'rank = 2'.
#[GET(path = "/hello/<name>/<age>")]
fn hi(name: &str, age: &str) -> String {
    format!("Hi {}! You age ({}) is kind of funky.", name, age)
}

fn main() {
    Rocket::new("localhost", 8000).mount_and_launch("/", routes![hello, hi]);
}
