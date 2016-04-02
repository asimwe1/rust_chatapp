#![feature(str_char)]
#![feature(specialization)]

extern crate term_painter;
extern crate hyper;

mod method;
mod error;
mod param;
mod router;
mod rocket;

pub mod request;
pub mod response;

pub mod handler {
    use super::{Request, Response};

    pub type Handler<'a> = fn(Request) -> Response<'a>;
}

pub use request::Request;
pub use method::Method;
pub use response::{Response, Responder};
pub use error::Error;
pub use param::FromParam;
pub use router::{Router, Route};
pub use rocket::Rocket;
pub use handler::Handler;
