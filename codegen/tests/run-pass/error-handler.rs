#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;

use rocket::{Error, Request};

#[error(404)]
fn err0() -> &'static str { "hi" }

#[error(404)]
fn err1a(_err: Error) -> &'static str { "hi" }

#[error(404)]
fn err1b(_req: &Request) -> &'static str { "hi" }

#[error(404)]
fn err2(_err: Error, _req: &Request) -> &'static str { "hi" }

fn main() {
    rocket::ignite()
        .catch(errors![err0, err1a, err1b, err2]);
}

