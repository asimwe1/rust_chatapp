#![feature(proc_macro_hygiene)]
#![allow(dead_code)] // This test is only here so that we can ensure it compiles.

#[macro_use] extern crate rocket;

use rocket::{Request, State};
use rocket::futures::future::BoxFuture;
use rocket::response::{Responder, Result};

struct SomeState;

pub struct CustomResponder<'r, R> {
    responder: R,
    state: &'r SomeState,
}

impl<'r, R: Responder<'r>> Responder<'r> for CustomResponder<'r, R> {
    fn respond_to<'a, 'x>(self, _: &'r Request<'a>) -> BoxFuture<'x, Result<'r>>
        where 'a: 'x, 'r: 'x, Self: 'x
    {
        unimplemented!()
    }
}

#[get("/unit_state")]
fn unit_state(state: State<SomeState>) -> CustomResponder<()> {
    CustomResponder { responder: (), state: state.inner() }
}

#[get("/string_state")]
fn string_state(state: State<SomeState>) -> CustomResponder<String> {
    CustomResponder { responder: "".to_string(), state: state.inner() }
}
