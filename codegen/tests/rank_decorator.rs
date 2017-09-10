#![feature(plugin, decl_macro)]
#![plugin(rocket_codegen)]

extern crate rocket;

#[get("/", rank = 1)]
fn get1() -> &'static str { "hi" }

#[get("/", rank = 2)]
fn get2() -> &'static str { "hi" }

#[get("/", rank = 3)]
fn get3() -> &'static str { "hi" }

#[test]
fn main() { }
