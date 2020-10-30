#[macro_use] extern crate rocket;

#[cfg(test)] mod tests;

use std::{io, env};
use rocket::data::{Capped, TempFile};

#[post("/upload", data = "<file>")]
async fn upload(mut file: Capped<TempFile<'_>>) -> io::Result<String> {
    file.persist_to(env::temp_dir().join("upload.txt")).await?;
    Ok(format!("{} bytes at {}", file.n.written, file.path().unwrap().display()))
}

#[get("/")]
fn index() -> &'static str {
    "Upload your text files by POSTing them to /upload.\n\
    Try `curl --data-binary @file.txt http://127.0.0.1:8000/upload`."
}

#[launch]
fn rocket() -> rocket::Rocket {
    rocket::ignite().mount("/", routes![index, upload])
}
