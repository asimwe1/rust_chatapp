use super::rocket;
use rocket::testing::MockRequest;
use rocket::http::Method::*;

#[test]
fn rewrite_get_put() {
    let rocket = rocket();
    let mut req = MockRequest::new(Get, "/");
    let mut response = req.dispatch_with(&rocket);
    assert_eq!(response.body_string(), Some("Hello, fairings!".into()));
}

#[test]
fn counts() {
    let rocket = rocket();

    // Issue 1 GET request.
    let mut req = MockRequest::new(Get, "/");
    req.dispatch_with(&rocket);

    // Check the GET count, taking into account _this_ GET request.
    let mut req = MockRequest::new(Get, "/counts");
    let mut response = req.dispatch_with(&rocket);
    assert_eq!(response.body_string(), Some("Get: 2\nPost: 0".into()));

    // Issue 1 more GET request and a POST.
    let mut req = MockRequest::new(Get, "/");
    req.dispatch_with(&rocket);
    let mut req = MockRequest::new(Post, "/");
    req.dispatch_with(&rocket);

    // Check the counts.
    let mut req = MockRequest::new(Get, "/counts");
    let mut response = req.dispatch_with(&rocket);
    assert_eq!(response.body_string(), Some("Get: 4\nPost: 1".into()));
}

#[test]
fn token() {
    let rocket = rocket();

    // Ensure the token is '123', which is what we have in `Rocket.toml`.
    let mut req = MockRequest::new(Get, "/token");
    let mut res = req.dispatch_with(&rocket);
    assert_eq!(res.body_string(), Some("123".into()));
}
