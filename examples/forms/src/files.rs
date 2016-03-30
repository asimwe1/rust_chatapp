use rocket;
use std::fs::File;
use std::io::Error as IOError;

#[route(GET, path = "/")]
pub fn index() -> File {
    File::open("static/index.html").unwrap()
}

#[route(GET, path = "/<file>")]
pub fn files(file: &str) -> Result<File, IOError> {
    File::open(format!("static/{}", file))
}
