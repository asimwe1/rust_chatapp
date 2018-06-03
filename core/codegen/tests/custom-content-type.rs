#![feature(plugin, decl_macro, custom_derive)]
#![plugin(rocket_codegen)]

extern crate rocket;

#[post("/", format = "application/x-custom")]
fn get() -> &'static str { "hi" }

#[test]
fn main() { }
