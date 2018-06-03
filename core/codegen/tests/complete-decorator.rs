#![feature(plugin, decl_macro, custom_derive)]
#![plugin(rocket_codegen)]
#![allow(dead_code, unused_variables)]

extern crate rocket;

use rocket::http::{Cookies, RawStr};
use rocket::request::Form;

#[derive(FromForm)]
struct User<'a> {
    name: &'a RawStr,
    nickname: String,
}

#[post("/<name>?<_query>", format = "application/json", data = "<user>", rank = 2)]
fn get<'r>(name: &RawStr,
           _query: User<'r>,
           user: Form<'r, User<'r>>,
           cookies: Cookies)
           -> &'static str {
    "hi"
}

#[test]
fn main() {
    let _ = routes![get];
}
