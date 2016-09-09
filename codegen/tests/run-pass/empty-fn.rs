#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;

#[get("")]
fn get() -> &'static str { "hi" }

fn main() {
    let _ = routes![get];
}
