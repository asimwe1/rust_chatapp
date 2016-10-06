mod empty;
mod responder;
mod redirect;
mod with_status;
mod outcome;
mod flash;
mod named_file;
mod stream;

pub mod data;

pub use self::responder::Responder;
pub use self::empty::{Empty, Forward};
pub use self::redirect::Redirect;
pub use self::with_status::StatusResponse;
pub use self::outcome::Outcome;
pub use self::flash::Flash;
pub use self::named_file::NamedFile;
pub use self::stream::Stream;
pub use self::data::Content;

use std::ops::{Deref, DerefMut};
use http::hyper::StatusCode;

pub struct Response<'a>(Box<Responder + 'a>);

impl<'a> Response<'a> {
    pub fn new<T: Responder + 'a>(body: T) -> Response<'a> {
        Response(Box::new(body))
    }

    pub fn with_status<T: Responder + 'a>(status: StatusCode,
                                          body: T)
                                          -> Response<'a> {
        Response(Box::new(StatusResponse::new(status, body)))
    }

    pub fn forward() -> Response<'a> {
        Response(Box::new(Forward))
    }

    pub fn with_raw_status<T: Responder + 'a>(status: u16, body: T) -> Response<'a> {
        let status_code = StatusCode::from_u16(status);
        Response(Box::new(StatusResponse::new(status_code, body)))
    }

    pub fn empty() -> Response<'a> {
        Response(Box::new(Empty::new(StatusCode::Ok)))
    }

    pub fn not_found() -> Response<'a> {
        Response(Box::new(Empty::new(StatusCode::NotFound)))
    }

    pub fn server_error(reason: &str) -> Response<'a> {
        warn_!("internal server error: {}", reason);
        Response(Box::new(Empty::new(StatusCode::InternalServerError)))
    }

    pub fn bad_request(reason: &str) -> Response<'a> {
        warn_!("bad request from user: {}", reason);
        Response(Box::new(Empty::new(StatusCode::BadRequest)))
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
