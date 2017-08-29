#![feature(plugin, decl_macro)]
#![plugin(rocket_codegen)]

extern crate rocket;

use rocket::{Error, Request};

#[error(404)]
fn err_a(_a: Error, _b: Request, _c: Error) -> &'static str { "hi" }
//~^ ERROR: can have at most 2

#[error(404)]
fn err_b(_a: (isize, usize)) -> &'static str { "hi" }
//~^ ERROR: unexpected error handler argument

fn main() {
}

