#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;

#[get("/")]
fn hello() -> &'static str {
    "Hello, world!"
}

fn main() {
    rocket::ignite().mount("/", routes![hello]).launch()
}

#[cfg(test)]
mod test {
    use super::rocket::{Rocket, Request};
    use super::rocket::http::Method;

    fn run_test<F>(f: F) where F: Fn(Rocket) {
        let rocket = Rocket::ignite().mount("/", routes![super::hello]);
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
