use std::fs::File;
use std::io;
use rocket;

#[route(GET, path = "/<top>/<file>")]
fn all_level_one(top: &str, file: &str) -> io::Result<File> {
    let file = format!("static/{}/{}", top, file);
    File::open(file)
}

#[route(GET, path = "/<file>")]
fn all(file: &str) -> io::Result<File> {
    let file = format!("static/{}", file);
    File::open(file)
}

pub fn routes() -> Vec<rocket::Route> {
    routes![all_level_one, all]
}
