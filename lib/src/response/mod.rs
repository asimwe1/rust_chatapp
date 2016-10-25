mod responder;
mod redirect;
mod with_status;
mod flash;
mod named_file;
mod stream;

pub mod data;

pub use self::responder::{Outcome, Responder};
pub use self::redirect::Redirect;
pub use self::with_status::StatusResponse;
pub use self::flash::Flash;
pub use self::named_file::NamedFile;
pub use self::stream::Stream;
pub use self::data::Content;

use outcome;
use request::Data;
use http::hyper::StatusCode;
use outcome::Outcome::*;

pub type Response<'a> = outcome::Outcome<Box<Responder + 'a>, StatusCode, Data>;

impl<'a> Response<'a> {
    #[inline(always)]
    pub fn success<T: Responder + 'a>(responder: T) -> Response<'a> {
        Success(Box::new(responder))
    }

    #[inline(always)]
    pub fn failure(code: StatusCode) -> Response<'static> {
        Failure(code)
    }

    #[inline(always)]
    pub fn forward(data: Data) -> Response<'static> {
        Forward(data)
    }

    #[inline(always)]
    pub fn with_raw_status<T: Responder + 'a>(status: u16, body: T) -> Response<'a> {
        let status_code = StatusCode::from_u16(status);
        Response::success(StatusResponse::new(status_code, body))
    }

    #[doc(hidden)]
    #[inline(always)]
    pub fn responder(self) -> Option<Box<Responder + 'a>> {
        self.succeeded()
    }
}
