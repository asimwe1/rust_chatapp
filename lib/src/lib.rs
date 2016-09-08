#![feature(question_mark)]
#![feature(specialization)]

extern crate term_painter;
extern crate hyper;
extern crate url;
extern crate mime;
#[macro_use] extern crate log;

#[macro_use]
pub mod logger;
pub mod form;
pub mod request;
pub mod response;
pub mod content_type;

mod method;
mod error;
mod param;
mod router;
mod rocket;
mod codegen;
mod catcher;

pub mod handler {
    use super::{Request, Response, Error};

    pub type Handler = for<'r> fn(&'r Request<'r>) -> Response<'r>;
    pub type ErrorHandler = for<'r> fn(error: Error, &'r Request<'r>) -> Response<'r>;
}

pub use logger::{RocketLogger, LoggingLevel};
pub use content_type::ContentType;
pub use codegen::{StaticRouteInfo, StaticCatchInfo};
pub use request::Request;
pub use method::Method;
pub use response::{Response, Responder};
pub use error::Error;
pub use param::{FromParam, FromSegments};
pub use router::{Router, Route};
pub use catcher::Catcher;
pub use rocket::Rocket;
pub use handler::{Handler, ErrorHandler};
