#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]

extern crate rocket;

use rocket::http::Cookies;
use rocket::request::Form;

#[derive(FromForm)]
struct User {
    name: String
}

#[post("/<name>?<query>", format = "application/json", data = "<user>", rank = 2)]
fn get(name: &str, query: User, user: Form<User>, cookies: &Cookies) -> &'static str { "hi" }

fn main() {
    let _ = routes![get];
}
