#![feature(plugin)]
#![plugin(rocket_macros)]

#[get("/<name>")] //~ ERROR 'name' is declared
fn get(_: &str) -> &'static str { "hi" } //~ ERROR isn't in the function

fn main() {  }
