mod responder;
mod redirect;
mod with_status;
mod flash;
mod named_file;
mod stream;
mod failure;

pub mod data;

pub use self::responder::Responder;
pub use self::redirect::Redirect;
pub use self::with_status::StatusResponse;
pub use self::flash::Flash;
pub use self::named_file::NamedFile;
pub use self::stream::Stream;
pub use self::data::Content;
pub use self::failure::Failure;
pub use outcome::Outcome;

use std::fmt;
use request::Data;
use http::hyper::{StatusCode, FreshHyperResponse};
use term_painter::Color::*;
use term_painter::ToStyle;

pub type ResponseOutcome<'a> = Outcome<(StatusCode, FreshHyperResponse<'a>)>;

pub enum Response<'a> {
    Forward(Data),
    Complete(Box<Responder + 'a>)
}

impl<'a> Response<'a> {
    #[inline(always)]
    pub fn complete<T: Responder + 'a>(body: T) -> Response<'a> {
        Response::Complete(Box::new(body))
    }

    #[inline(always)]
    pub fn forward(data: Data) -> Response<'static> {
        Response::Forward(data)
    }

    #[inline(always)]
    pub fn failed(code: StatusCode) -> Response<'static> {
        Response::complete(Failure::new(code))
    }

    #[inline(always)]
    pub fn with_raw_status<T: Responder + 'a>(status: u16, body: T) -> Response<'a> {
        let status_code = StatusCode::from_u16(status);
        Response::complete(StatusResponse::new(status_code, body))
    }

    #[doc(hidden)]
    #[inline(always)]
    pub fn responder(self) -> Option<Box<Responder + 'a>> {
        match self {
            Response::Complete(responder) => Some(responder),
            _ => None
        }
    }
}

impl<'a> fmt::Display for Response<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Response::Complete(..) => write!(f, "{}", Green.paint("Complete")),
            Response::Forward(..) => write!(f, "{}", Yellow.paint("Forwarding")),
        }
    }
}
