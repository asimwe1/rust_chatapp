#![feature(plugin)]
#![plugin(rocket_macros)]

extern crate rocket;
use rocket::Rocket;

#[get("/")]
fn hello() -> &'static str {
    "Hello, world!"
}

fn main() {
    Rocket::new("localhost", 8000).mount_and_launch("/", routes![hello]);
}

#[cfg(test)]
mod test {
    use super::rocket::{Rocket, Request, Method};

    fn run_test<F>(f: F) where F: Fn(Rocket) {
        let mut rocket = Rocket::new("_", 0);
        rocket.mount("/", routes![super::hello]);
        f(rocket);
    }

    #[test]
    fn test_hello() {
        run_test(|_rocket: Rocket| {
            let _req = Request::mock(Method::Get, "/");
            // TODO: Allow something like this:
            // let result = rocket.route(&req);
            // assert_eq!(result.as_str(), "Hello, world!")
        });
    }
}
