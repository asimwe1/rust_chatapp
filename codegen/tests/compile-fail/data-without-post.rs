#![feature(plugin, decl_macro)]
#![plugin(rocket_codegen)]

extern crate rocket;

use rocket::Data;

#[get("/", data = "<something>")]
//~^ ERROR payload supporting methods
fn get(something: Data) -> &'static str { "hi" }

fn main() {  }
