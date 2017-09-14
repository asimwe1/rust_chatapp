#![feature(plugin, decl_macro)]
#![plugin(rocket_codegen)]
#![allow(dead_code, unused_variables)]

extern crate rocket;

use rocket::http::RawStr;
use rocket::request::FromParam;

struct S;

impl<'a> FromParam<'a> for S {
    type Error = ();
    fn from_param(param: &'a RawStr) -> Result<Self, Self::Error> { Ok(S) }
}

#[post("/<id>")]
fn simple(id: i32) -> &'static str { "" }

#[post("/<id>/<name>")]
fn not_uri_display(id: i32, name: S) -> &'static str { "" }

#[post("/<id>/<name>")]
fn not_uri_display_but_unused(id: i32, name: S) -> &'static str { "" }

fn main() {
    uri!(simple: id = "hi");
        //~^ ERROR trait bound `i32: rocket::http::uri::FromUriParam<&str>` is not satisfied
    uri!(simple: "hello");
        //~^ ERROR trait bound `i32: rocket::http::uri::FromUriParam<&str>` is not satisfied
    uri!(simple: id = 239239i64);
        //~^ ERROR trait bound `i32: rocket::http::uri::FromUriParam<i64>` is not satisfied
    uri!(not_uri_display: 10, S);
        //~^ ERROR trait bound `S: rocket::http::uri::FromUriParam<_>` is not satisfied
}
