#![feature(plugin, decl_macro)]
#![plugin(rocket_codegen)]

extern crate rocket;

#[get] //~ ERROR incorrect use of attribute
//~^ ERROR malformed attribute
fn get() -> &'static str { "hi" }

fn main() {
    let _ = routes![get];
}

