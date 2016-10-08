#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]

extern crate rocket;

use rocket::http::Cookies;

#[derive(FromForm)]
struct User {
    name: String
}

#[post("/<name>?<query>", format = "application/json", form = "<user>", rank = 2)]
fn get(name: &str, query: User, user: User, cookies: &Cookies) -> &'static str { "hi" }

fn main() {
    let _ = routes![get];
}
