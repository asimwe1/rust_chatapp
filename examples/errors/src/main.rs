#![feature(plugin)]
#![plugin(rocket_macros)]

extern crate rocket;
use rocket::Rocket;

#[route(GET, path = "/hello/<name>/<age>")]
fn hello(name: &str, age: i8) -> String {
    format!("Hello, {} year old named {}!", age, name)
}

#[error(code = "404")]
fn not_found() -> &'static str {
    "Sorry, I couldn't find what you're looking for."
}

fn main() {
    let mut rocket = Rocket::new("localhost", 8000);
    rocket.mount("/", routes![hello]);
    rocket.catch(errors![not_found]);
    // rocket.catch_and_launch(errors![not_found]);
    rocket.launch();
}
