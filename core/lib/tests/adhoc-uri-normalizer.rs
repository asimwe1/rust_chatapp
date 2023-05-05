#[macro_use] extern crate rocket;

use std::path::PathBuf;

use rocket::local::blocking::Client;
use rocket::fairing::AdHoc;

#[get("/foo")]
fn foo() -> &'static str { "foo" }

#[get("/bar")]
fn not_bar() -> &'static str { "not_bar" }

#[get("/bar/")]
fn bar() -> &'static str { "bar" }

#[get("/foo/<_>/<_baz..>")]
fn baz(_baz: PathBuf) -> &'static str { "baz" }

#[get("/doggy/<_>/<_baz..>?doggy")]
fn doggy(_baz: PathBuf) -> &'static str { "doggy" }

#[test]
fn test_adhoc_normalizer_works_as_expected () {
    let rocket = rocket::build()
        .mount("/", routes![foo, bar, not_bar, baz, doggy])
        .mount("/base", routes![foo, bar, not_bar, baz, doggy])
        .attach(AdHoc::uri_normalizer());

    let client = Client::debug(rocket).unwrap();

    let response = client.get("/foo/").dispatch();
    assert_eq!(response.into_string().unwrap(), "foo");

    let response = client.get("/foo").dispatch();
    assert_eq!(response.into_string().unwrap(), "foo");

    let response = client.get("/bar/").dispatch();
    assert_eq!(response.into_string().unwrap(), "bar");

    let response = client.get("/bar").dispatch();
    assert_eq!(response.into_string().unwrap(), "not_bar");

    let response = client.get("/foo/bar").dispatch();
    assert_eq!(response.into_string().unwrap(), "baz");

    let response = client.get("/doggy/bar?doggy").dispatch();
    assert_eq!(response.into_string().unwrap(), "doggy");

    let response = client.get("/foo/bar/").dispatch();
    assert_eq!(response.into_string().unwrap(), "baz");

    let response = client.get("/foo/bar/baz").dispatch();
    assert_eq!(response.into_string().unwrap(), "baz");

    let response = client.get("/base/foo/").dispatch();
    assert_eq!(response.into_string().unwrap(), "foo");

    let response = client.get("/base/foo").dispatch();
    assert_eq!(response.into_string().unwrap(), "foo");

    let response = client.get("/base/bar/").dispatch();
    assert_eq!(response.into_string().unwrap(), "bar");

    let response = client.get("/base/bar").dispatch();
    assert_eq!(response.into_string().unwrap(), "not_bar");

    let response = client.get("/base/foo/bar").dispatch();
    assert_eq!(response.into_string().unwrap(), "baz");

    let response = client.get("/doggy/foo/bar?doggy").dispatch();
    assert_eq!(response.into_string().unwrap(), "doggy");

    let response = client.get("/base/foo/bar/").dispatch();
    assert_eq!(response.into_string().unwrap(), "baz");

    let response = client.get("/base/foo/bar/baz").dispatch();
    assert_eq!(response.into_string().unwrap(), "baz");
}
