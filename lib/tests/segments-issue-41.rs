#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;

use std::path::PathBuf;

#[get("/test/<path..>")]
fn test(path: PathBuf) -> String {
    format!("{:?}", path)
}

#[get("/two/<path..>")]
fn two(path: PathBuf) -> String {
    format!("{:?}", path)
}

#[get("/one/two/<path..>")]
fn one_two(path: PathBuf) -> String {
    format!("{:?}", path)
}

#[get("/<path..>", rank = 2)]
fn none(path: PathBuf) -> String {
    format!("{:?}", path)
}

use rocket::testing::MockRequest;
use rocket::http::Method::*;

#[test]
fn segments_works() {
    let rocket = rocket::ignite().mount("/", routes![test, two, one_two, none]);

    // We construct a path that matches each of the routes above. We ensure the
    // prefix is stripped, confirming that dynamic segments are working.
    for prefix in &["", "/test", "/two", "/one/two"] {
        let path = "this/is/the/path/we/want";
        let mut req = MockRequest::new(Get, format!("{}/{}", prefix, path));

        let mut response = req.dispatch_with(&rocket);
        let body_str = response.body().and_then(|b| b.into_string());
        assert_eq!(body_str, Some(format!("{:?}", path)));
    }
}
