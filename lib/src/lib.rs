#![feature(question_mark)]
#![feature(specialization)]

extern crate term_painter;
extern crate hyper;

mod method;
mod error;
mod param;
mod router;
mod rocket;
mod codegen;
mod catcher;

pub mod form;
pub mod request;
pub mod response;

pub mod handler {
    use super::{Request, Response};

    pub type Handler = for<'r> fn(Request<'r>) -> Response<'r>;
}

pub use codegen::{StaticRouteInfo, StaticCatchInfo};
pub use request::Request;
pub use method::Method;
pub use response::{Response, Responder};
pub use error::Error;
pub use param::FromParam;
pub use router::{Router, Route};
pub use catcher::Catcher;
pub use rocket::Rocket;
pub use handler::Handler;
