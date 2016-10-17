#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;

#[cfg(test)] mod tests;

#[get("/hello/<name>/<age>")]
fn hello(name: &str, age: i8) -> String {
    format!("Hello, {} year old named {}!", age, name)
}

#[get("/hello/<name>/<age>", rank = 2)]
fn hi(name: &str, age: &str) -> String {
    format!("Hi {}! Your age ({}) is kind of funky.", name, age)
}

fn main() {
    rocket::ignite().mount("/", routes![hi, hello]).launch();
}
