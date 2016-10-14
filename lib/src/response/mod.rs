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

use request::Data;
use http::hyper::{StatusCode, FreshHyperResponse};

pub type ResponseOutcome<'a> = Outcome<(), (), (StatusCode, FreshHyperResponse<'a>)>;

impl<'a> ResponseOutcome<'a> {
    #[inline(always)]
    pub fn of<A, B>(result: Result<A, B>) -> Self {
        match result {
            Ok(_) => Outcome::Success(()),
            Err(_) => Outcome::Failure(())
        }
    }

    #[inline(always)]
    pub fn success() -> ResponseOutcome<'a> {
        Outcome::Success(())
    }

    #[inline(always)]
    pub fn failure() -> ResponseOutcome<'a> {
        Outcome::Failure(())
    }

    #[inline(always)]
    pub fn forward(s: StatusCode, r: FreshHyperResponse<'a>) -> ResponseOutcome<'a> {
        Outcome::Forward((s, r))
    }
}

pub type Response<'a> = Outcome<Box<Responder + 'a>, StatusCode, Data>;

impl<'a> Response<'a> {
    #[inline(always)]
    pub fn success<T: Responder + 'a>(responder: T) -> Response<'a> {
        Outcome::Success(Box::new(responder))
    }

    #[inline(always)]
    pub fn failure(code: StatusCode) -> Response<'static> {
        Outcome::Failure(code)
    }

    #[inline(always)]
    pub fn forward(data: Data) -> Response<'static> {
        Outcome::Forward(data)
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
