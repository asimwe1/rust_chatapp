use data::Data;
use request::Request;
use response::{self, Response, Responder};
use error::Error;
use http::Status;
use outcome;

/// Type alias for the `Outcome` of a `Handler`.
pub type Outcome<'r> = outcome::Outcome<Response<'r>, Status, Data>;

impl<'r> Outcome<'r> {
    #[inline]
    pub fn of<T: Responder<'r>>(responder: T) -> Outcome<'r> {
        match responder.respond() {
            Ok(response) => outcome::Outcome::Success(response),
            Err(status) => outcome::Outcome::Failure(status)
        }
    }

    #[inline(always)]
    pub fn failure(code: Status) -> Outcome<'static> {
        outcome::Outcome::Failure(code)
    }

    #[inline(always)]
    pub fn forward(data: Data) -> Outcome<'static> {
        outcome::Outcome::Forward(data)
    }
}

/// The type of a request handler.
pub type Handler = for<'r> fn(&'r Request, Data) -> Outcome<'r>;

/// The type of an error handler.
pub type ErrorHandler = for<'r> fn(Error, &'r Request) -> response::Result<'r>;
