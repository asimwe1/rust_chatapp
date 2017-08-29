#![feature(plugin, decl_macro)]
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
fn err2a(_err: Error, _req: &Request) -> &'static str { "hi" }

#[error(404)]
fn err2b<'a>(_err: Error, _req: &'a Request) -> &'a str { "hi" }

fn main() {
}

