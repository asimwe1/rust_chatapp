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
//! documentation](../rocket_contrib), which contains JSON and templating
//! support.

extern crate term_painter;
extern crate hyper;
extern crate url;
extern crate mime;
#[macro_use] extern crate log;

#[doc(hidden)]
#[macro_use]
pub mod logger;
pub mod form;
pub mod request;
pub mod response;
#[doc(hidden)]
pub mod content_type;

mod method;
mod error;
mod router;
mod rocket;
mod codegen;
mod catcher;

/// Defines the types for request and error handlers.
pub mod handler {
    use super::{Request, Response, Error};

    /// The type of a request handler.
    pub type Handler = for<'r> fn(&'r Request<'r>) -> Response<'r>;

    /// The type of an error handler.
    pub type ErrorHandler = for<'r> fn(error: Error, &'r Request<'r>) -> Response<'r>;
}

pub use content_type::ContentType;
pub use codegen::{StaticRouteInfo, StaticCatchInfo};
pub use request::Request;
pub use method::Method;
#[doc(inline)]
pub use response::{Response, Responder};
pub use error::Error;
pub use router::{Router, Route};
pub use catcher::Catcher;
pub use rocket::Rocket;
#[doc(inline)]
pub use handler::{Handler, ErrorHandler};
#[doc(inline)]
pub use logger::LoggingLevel;
