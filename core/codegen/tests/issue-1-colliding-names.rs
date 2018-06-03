#![feature(plugin, decl_macro)]
#![plugin(rocket_codegen)]

extern crate rocket;

#[get("/<todo>")]
fn todo(todo: String) -> String {
    todo
}

#[test]
fn main() {
    let _ = routes![todo];
}
