use super::rocket;
use rocket::testing::MockRequest;
use rocket::http::Method::*;

#[test]
fn hello_world() {
    let rocket = rocket::ignite().mount("/", routes![super::hello]);
    let req = MockRequest::new(Get, "/");
    let result = req.dispatch_with(&rocket);
    assert_eq!(result.unwrap().as_str(), "Hello, world!");
}
