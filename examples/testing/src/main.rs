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
    use super::rocket;
    use rocket::testing::MockRequest;
    use rocket::http::Method::*;

    #[test]
    fn test_hello() {
        let rocket = rocket::ignite().mount("/", routes![super::hello]);
        let mut req = MockRequest::new(Get, "/");
        let mut response = req.dispatch_with(&rocket);

        let body_string = response.body().and_then(|b| b.to_string());
        assert_eq!(body_string, Some("Hello, world!".to_string()));
    }
}
