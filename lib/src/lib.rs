#![feature(str_char)]
#![feature(specialization)]

extern crate term_painter;
extern crate hyper;

mod method;
mod error;
mod param;
mod router;
mod rocket;
mod route;

pub mod request;
pub mod response;

pub use request::Request;
pub use method::Method;
pub use response::{Response, Responder};
pub use error::Error;
pub use param::FromParam;
pub use router::Router;
pub use route::{Route, Handler};
pub use rocket::Rocket;
