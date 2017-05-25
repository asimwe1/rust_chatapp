use rocket::testing::MockRequest;
use rocket::http::{Status, ContentType};
use rocket::http::Method::*;

use std::io::Read;
use std::fs::File;

#[test]
fn test_index() {
    let rocket = super::rocket();
    let mut req = MockRequest::new(Get, "/");
    let mut res = req.dispatch_with(&rocket);

    assert_eq!(res.body_string(), Some(super::index().to_string()));
}

#[test]
fn test_raw_upload() {
    const UPLOAD_CONTENTS: &str = "Hey! I'm going to be uploaded. :D Yay!";

    let rocket = super::rocket();
    let mut req = MockRequest::new(Post, "/upload")
        .header(ContentType::Plain)
        .body(UPLOAD_CONTENTS);

    // Do the upload. Make sure we get the expected results.
    let mut res = req.dispatch_with(&rocket);
    assert_eq!(res.status(), Status::Ok);
    assert_eq!(res.body_string(), Some(UPLOAD_CONTENTS.len().to_string()));

    // Ensure we find the body in the /tmp/upload.txt file.
    let mut file_contents = String::new();
    let mut file = File::open("/tmp/upload.txt").expect("open upload.txt file");
    file.read_to_string(&mut file_contents).expect("read upload.txt");
    assert_eq!(&file_contents, UPLOAD_CONTENTS);
}
