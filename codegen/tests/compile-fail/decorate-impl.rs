#![feature(plugin, decl_macro)]
#![plugin(rocket_codegen)]

extern crate rocket;

#[get("")]        //~ ERROR can only be used on functions
impl C for A {  } //~ ERROR but was applied

fn main() {
    let _ = routes![get];
}

