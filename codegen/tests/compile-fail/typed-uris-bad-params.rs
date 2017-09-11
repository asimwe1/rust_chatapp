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
    uri!(simple); //~ ERROR expects 1 parameter but 0 were supplied
    uri!(simple: 1, 23); //~ ERROR expects 1 parameter but 2 were supplied
    uri!(simple: "Hello", 23, ); //~ ERROR expects 1 parameter but 2 were supplied
    uri!(guard_1: "hi", 100); //~ ERROR expects 1 parameter but 2 were supplied

    uri!(simple: id = 100, name = "hi"); //~ ERROR invalid parameters
    uri!(simple: id = 100, id = 100); //~ ERROR invalid parameters
    uri!(simple: name = 100, id = 100); //~ ERROR invalid parameters
    uri!(simple: id = 100, id = 100, ); //~ ERROR invalid parameters
    uri!(simple: name = "hi"); //~ ERROR invalid parameters
    uri!(guard_1: cookies = "hi", id = 100); //~ ERROR invalid parameters
    uri!(guard_1: id = 100, cookies = "hi"); //~ ERROR invalid parameters
}
