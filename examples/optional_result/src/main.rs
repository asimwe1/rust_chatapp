#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;

#[get("/users/<name>")]
fn user(name: &str) -> Option<&'static str> {
    if name == "Sergio" {
        Some("Hello, Sergio!")
    } else {
        None
    }
}

fn main() {
    rocket::ignite().mount("/", routes![user]).launch();
}
