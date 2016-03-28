#![feature(plugin)]
#![plugin(rocket_macros)]

extern crate rocket;

use rocket::Rocket;
use std::fs::File;
use std::io::Error as IOError;
use rocket::response::Redirect;

#[route(GET, path = "/")]
fn index() -> File {
    File::open("static/index.html").unwrap()
}

#[route(GET, path = "/<file>")]
fn files(file: &str) -> Result<File, IOError> {
    File::open(format!("static/{}", file))
}

#[route(GET, path = "/user/<username>")]
fn user_page(username: &str) -> String {
    format!("This is {}'s page.", username)
}

// TODO: Actually look at form parameters.
#[route(POST, path = "/login")]
fn login() -> Result<Redirect, &'static str> {
    if true {
        Ok(Redirect::other("/user/some_name"))
    } else {
        Err("Sorry, the username and password are invalid.")
    }
}

fn main() {
    let rocket = Rocket::new("localhost", 8000);
    rocket.mount_and_launch("/", routes![index, files, user_page, login]);
}
