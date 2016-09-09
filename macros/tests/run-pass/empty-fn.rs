#![feature(plugin)]
#![plugin(rocket_macros)]

extern crate rocket;

#[get("")]
fn get() -> &'static str { "hi" }

fn main() {
    let _ = routes![get];
}
