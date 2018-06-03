#![feature(plugin, decl_macro)]
#![plugin(rocket_codegen)]
#![allow(dead_code, unused_variables)]

extern crate rocket;

#[get("/test/<one>/<two>/<three>")]
fn get(one: String, two: usize, three: isize) -> &'static str { "hi" }

#[get("/test/<_one>/<_two>/<__three>")]
fn ignored(_one: String, _two: usize, __three: isize) -> &'static str { "hi" }

#[test]
fn main() {
    let _ = routes![get, ignored];
}
