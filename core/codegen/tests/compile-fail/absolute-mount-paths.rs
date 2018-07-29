#![feature(plugin, decl_macro)]
#![plugin(rocket_codegen)]

#[get("a")] //~ ERROR invalid
fn get() -> &'static str { "hi" }

#[get("")] //~ ERROR invalid
fn get1(id: usize) -> &'static str { "hi" }

#[get("a/b/c")] //~ ERROR invalid
fn get2(id: usize) -> &'static str { "hi" }

fn main() {  }
