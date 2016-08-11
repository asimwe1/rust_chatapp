#![feature(plugin)]
#![plugin(rocket_macros)]

extern crate rocket;

use rocket::{Rocket, Error};

// One idea of what we could get.
// #[route(GET, path = "/hello?{name,age}")]
// fn hello(name: &str, age: &str) -> String {
//     "Hello!".to_string()
//     // format!("Hello, {} year old named {}!", age, name)
// }

// Another idea.
// #[route(GET, path = "/hello")]
// fn hello(q: QueryParams) -> IOResult<String> {
//     format!("Hello, {} year old named {}!", q.get("name")?, q.get("age")?)
// }

#[route(GET, path = "/hello")]
fn hello() -> &'static str {
    "Hello there! Don't have query params yet, but we're working on it."
}

fn main() {
    Rocket::new("localhost", 8000).mount_and_launch("/", routes![hello]);
}
