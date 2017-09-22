#![feature(plugin, decl_macro)]
#![plugin(rocket_codegen)]

extern crate rocket;

#[get("/", data = "<something>")]
fn get(something: rocket::Data) -> &'static str { "hi" }
