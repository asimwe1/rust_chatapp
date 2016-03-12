#![feature(plugin)]
#![plugin(rocket_macros)]

extern crate rocket;
use rocket::{Rocket, Request, Response, Method, Route};

#[route(GET, path = "/hello")]
fn hello() -> &'static str {
    "Hello, world!"
}

mod test {
    use rocket::{Request, Response, Method, Route};

    #[route(GET, path = "")]
    pub fn hello() -> &'static str {
        "Hello, world!"
    }
}

fn main() {
    let mut rocket = Rocket::new("localhost", 8000);
    rocket.mount("/test", routes![test::hello]);
    rocket.mount_and_launch("/", routes![hello]);
}
