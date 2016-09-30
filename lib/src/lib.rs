#![feature(question_mark)]
#![feature(specialization)]

//! # Rocket
//!
//! Hello, and welcome to the core Rocket API documentation!
//!
//! This API documentation is highly technical and is purely a reference.
//! There's an [overview](https://rocket.rs/overview) of Rocket on the main site
//! as well as a [full, detailed guide](https://rocket.rs/guide). If you'd like
//! pointers on getting started, see the
//! [quickstart](https://rocket.rs/guide/quickstart) or [getting started
//! chapter](https://rocket.rs/guide/getting_started) of the guide.
//!
//! You may also be interested in looking at the [contrib API
//! documentation](../rocket_contrib), which contains JSON and templaring
//! support.

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
#[doc(hidden)]
pub mod content_type;

mod method;
mod error;
mod param;
mod router;
mod rocket;
mod codegen;
mod catcher;

#[doc(hidden)]
pub mod handler {
    use super::{Request, Response, Error};

    pub type Handler = for<'r> fn(&'r Request<'r>) -> Response<'r>;
    pub type ErrorHandler = for<'r> fn(error: Error, &'r Request<'r>) -> Response<'r>;
}

#[doc(hidden)]
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
