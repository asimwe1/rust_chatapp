#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;

#[get("/hello/<name>/<age>")]
fn hello(name: &str, age: i8) -> String {
    format!("Hello, {} year old named {}!", age, name)
}

#[get("/hello/<name>")]
fn hi<'r>(name: &'r str) -> &'r str {
    name
}

fn main() {
    rocket::ignite().mount("/", routes![hello, hi]).launch();
}
