#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;

#[get("/<todo>")]
fn todo(todo: &str) -> &str {
    todo
}

fn main() {
    let _ = routes![todo];
}

