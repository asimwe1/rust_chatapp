#![feature(plugin, decl_macro, custom_derive)]
#![plugin(rocket_codegen)]
#![allow(dead_code, unused_variables)]

extern crate rocket;

use std::fmt;

use rocket::http::Cookies;

#[post("/<id>")]
fn simple(id: i32) -> &'static str { "" }

#[post("/<id>")]
fn guard_1(cookies: Cookies, id: i32) -> &'static str { "" }

fn main() {
    uri!(simple);
    uri!(simple: 1, 23);
    uri!(simple: "Hello", 23, );
    uri!(guard_1: "hi", 100);

    uri!(simple: id = 100, name = "hi");
    uri!(simple: id = 100, id = 100);
    uri!(simple: name = 100, id = 100);
    uri!(simple: id = 100, id = 100, );
    uri!(simple: name = "hi");
    uri!(guard_1: cookies = "hi", id = 100);
    uri!(guard_1: id = 100, cookies = "hi");
}
