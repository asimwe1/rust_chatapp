mod empty;
mod responder;
mod redirect;

pub use self::responder::Responder;
pub use self::empty::Empty;
pub use self::redirect::Redirect;

pub use hyper::server::Response as HypResponse;
pub use hyper::net::Fresh as HypFresh;
pub use hyper::status::StatusCode;
pub use hyper::header;

use std::ops::{Deref, DerefMut};

pub struct Response<'a>(Box<Responder + 'a>);

impl<'a> Response<'a> {
    pub fn new<T: Responder + 'a>(body: T) -> Response<'a> {
        Response(Box::new(body))
    }

    pub fn empty() -> Response<'a> {
        Response(Box::new(Empty::new(StatusCode::Ok)))
    }

    pub fn not_found() -> Response<'a> {
        Response(Box::new(Empty::new(StatusCode::NotFound)))
    }

    pub fn server_error() -> Response<'a> {
        Response(Box::new(Empty::new(StatusCode::InternalServerError)))
    }
}

impl<'a> Deref for Response<'a> {
    type Target = Box<Responder + 'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> DerefMut for Response<'a> {
    fn deref_mut(&mut self) -> &mut Box<Responder + 'a> {
        &mut self.0
    }
}
