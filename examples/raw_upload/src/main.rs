#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;

use rocket::request::Data;
use rocket::response::Failure;
use rocket::http::{StatusCode, ContentType};

#[post("/upload", data = "<data>")]
fn upload(content_type: ContentType, data: Data) -> Result<String, Failure> {
    if !content_type.is_text() {
        println!("    => Content-Type of upload must be text. Ignoring.");
        return Err(Failure(StatusCode::BadRequest));
    }

    match data.stream_to_file("/tmp/upload.txt") {
        Ok(n) => Ok(format!("OK: {} bytes uploaded.", n)),
        Err(e) => {
            println!("    => Failed writing to file: {:?}.", e);
            return Err(Failure(StatusCode::InternalServerError));
        }
    }
}

#[get("/")]
fn index() -> &'static str {
    "Upload your text files by POSTing them to /upload."
}

fn main() {
    rocket::ignite().mount("/", routes![index, upload]).launch();
}
