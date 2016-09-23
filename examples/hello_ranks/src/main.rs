#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
use rocket::Rocket;

#[get("/hello/<name>/<age>")]
fn hello(name: &str, age: i8) -> String {
    format!("Hello, {} year old named {}!", age, name)
}

#[get("/hello/<name>/<age>", rank = 2)]
fn hi(name: &str, age: &str) -> String {
    format!("Hi {}! Your age ({}) is kind of funky.", name, age)
}

fn main() {
    Rocket::new("localhost", 8000).mount_and_launch("/", routes![hi, hello]);
}
