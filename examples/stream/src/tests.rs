use std::fs::{self, File};
use std::io::prelude::*;

use rocket::testing::MockRequest;
use rocket::http::Method::*;

#[test]
fn test_root() {
    let rocket = super::rocket();
    let mut req = MockRequest::new(Get, "/");
    let mut res = req.dispatch_with(&rocket);

    // Check that we have exactly 25,000 'a'.
    let res_str = res.body_string().unwrap();
    assert_eq!(res_str.len(), 25000);
    for byte in res_str.as_bytes() {
        assert_eq!(*byte, b'a');
    }
}

#[test]
fn test_file() {
    // Create the 'big_file'
    const CONTENTS: &str = "big_file contents...not so big here";
    let mut file = File::create(super::FILENAME).expect("create big_file");
    file.write_all(CONTENTS.as_bytes()).expect("write to big_file");

    // Get the big file contents, hopefully.
    let rocket = super::rocket();
    let mut req = MockRequest::new(Get, "/big_file");
    let mut res = req.dispatch_with(&rocket);
    assert_eq!(res.body_string(), Some(CONTENTS.into()));

    // Delete the 'big_file'.
    fs::remove_file(super::FILENAME).expect("remove big_file");
}
