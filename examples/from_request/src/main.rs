#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;

use std::fmt;
use rocket::request::{Request, FromRequest};

#[derive(Debug)]
struct HeaderCount(usize);

impl fmt::Display for HeaderCount {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<'r, 'c> FromRequest<'r, 'c> for HeaderCount {
    type Error = ();
    fn from_request(request: &'r Request<'c>) -> Result<Self, Self::Error> {
        Ok(HeaderCount(request.headers().len()))
    }
}

#[get("/")]
fn header_count(header_count: HeaderCount) -> String {
    format!("Your request contained {} headers!", header_count)
}

fn main() {
    rocket::ignite().mount("/", routes![header_count]).launch()
}
