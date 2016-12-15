use super::rocket;
use rocket::testing::MockRequest;
use rocket::http::Method::*;

fn test(uri: &str, expected: String) {
    let rocket = rocket::ignite().mount("/", routes![super::hello, super::hi]);
    let result = MockRequest::new(Get, uri).dispatch_with(&rocket);
    assert_eq!(result.unwrap(), expected);
}

fn test_404(uri: &str) {
    let rocket = rocket::ignite().mount("/", routes![super::hello, super::hi]);
    let result = MockRequest::new(Get, uri).dispatch_with(&rocket);
    // FIXME: Be able to check that actual HTTP response status code.
    // assert!(result.unwrap().contains("404"));
    assert!(result.is_none());
}

#[test]
fn test_hello() {
    for &(name, age) in &[("Mike", 22), ("Michael", 80), ("A", 0), ("a", 127)] {
        test(&format!("/hello/{}/{}", name, age),
            format!("Hello, {} year old named {}!", age, name));
    }
}

#[test]
fn test_failing_hello() {
    test_404("/hello/Mike/1000");
    test_404("/hello/Mike/128");
    test_404("/hello/Mike/-129");
}

#[test]
fn test_hi() {
    for name in &["Mike", "A", "123", "hi", "c"] {
        test(&format!("/hello/{}", name), name.to_string());
    }
}
