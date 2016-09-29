#![feature(plugin)]
#![plugin(rocket_codegen)]

#[get("/<name>")] //~ ERROR 'name' is declared
fn get(other: &str) -> &'static str { "hi" } //~ ERROR isn't in the function

#[get("/a?<r>")] //~ ERROR 'r' is declared
fn get1() -> &'static str { "hi" } //~ ERROR isn't in the function

#[get("/a", form = "<test>")] //~ ERROR 'test' is declared
fn get2() -> &'static str { "hi" } //~ ERROR isn't in the function

fn main() {  }
