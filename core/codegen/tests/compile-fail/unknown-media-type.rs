// must-compile-successfully

#![feature(plugin, decl_macro)]
#![plugin(rocket_codegen)]

extern crate rocket;

#[get("/", format = "application/x-custom")] //~ WARNING not a known media type
fn one() -> &'static str { "hi" }

#[get("/", format = "x-custom/plain")] //~ WARNING not a known media type
fn two() -> &'static str { "hi" }

#[get("/", format = "x-custom/x-custom")] //~ WARNING not a known media type
fn three() -> &'static str { "hi" }

fn main() {  }
