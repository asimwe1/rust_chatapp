#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]

extern crate rocket;

use rocket::{Rocket, Error};

#[derive(FromForm)]
struct Person<'r> {
    name: &'r str,
    age: Option<u8>
}

#[get("/hello?<person>")]
fn hello(person: Person) -> String {
    if let Some(age) = person.age {
        format!("Hello, {} year old named {}!", age, person.name)
    } else {
        format!("Hello {}!", person.name)
    }
}

fn main() {
    Rocket::new("localhost", 8000).mount_and_launch("/", routes![hello]);
}
