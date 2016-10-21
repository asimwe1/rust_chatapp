use std::fmt::Debug;

use request::Request;
use outcome::Outcome;
use http::{StatusCode, ContentType, Method, Cookies};

/// Type alias for the `Outcome` of a `FromRequest` conversion.
pub type RequestOutcome<T, E> = Outcome<T, (StatusCode, E), ()>;

impl<T, E> RequestOutcome<T, E> {
    #[inline(always)]
    pub fn of(result: Result<T, E>) -> Self {
        match result {
            Ok(val) => Outcome::Success(val),
            Err(_) => Outcome::Forward(())
        }
    }

    #[inline(always)]
    pub fn success(t: T) -> Self {
        Outcome::Success(t)
    }

    #[inline(always)]
    pub fn failure(code: StatusCode, error: E) -> Self {
        Outcome::Failure((code, error))
    }

    #[inline(always)]
    pub fn forward() -> Self {
        Outcome::Forward(())
    }
}

pub trait FromRequest<'r>: Sized {
    type Error: Debug;

    fn from_request(request: &'r Request) -> RequestOutcome<Self, Self::Error>;
}

impl<'r> FromRequest<'r> for &'r Request {
    type Error = ();

    fn from_request(request: &'r Request) -> RequestOutcome<Self, Self::Error> {
        RequestOutcome::success(request)
    }
}

impl<'r> FromRequest<'r> for Method {
    type Error = ();

    fn from_request(request: &'r Request) -> RequestOutcome<Self, Self::Error> {
        RequestOutcome::success(request.method)
    }
}

impl<'r> FromRequest<'r> for &'r Cookies {
    type Error = ();

    fn from_request(request: &'r Request) -> RequestOutcome<Self, Self::Error> {
        RequestOutcome::success(request.cookies())
    }
}

impl<'r> FromRequest<'r> for ContentType {
    type Error = ();

    fn from_request(request: &'r Request) -> RequestOutcome<Self, Self::Error> {
        RequestOutcome::success(request.content_type())
    }
}

impl<'r, T: FromRequest<'r>> FromRequest<'r> for Result<T, T::Error> {
    type Error = ();

    fn from_request(request: &'r Request) -> RequestOutcome<Self, Self::Error> {
        match T::from_request(request) {
            Outcome::Success(val) => RequestOutcome::success(Ok(val)),
            Outcome::Failure((_, e)) => RequestOutcome::success(Err(e)),
            Outcome::Forward(_) => RequestOutcome::forward(),
        }
    }
}

impl<'r, T: FromRequest<'r>> FromRequest<'r> for Option<T> {
    type Error = ();

    fn from_request(request: &'r Request) -> RequestOutcome<Self, Self::Error> {
        match T::from_request(request) {
            Outcome::Success(val) => RequestOutcome::success(Some(val)),
            Outcome::Failure(_) => RequestOutcome::success(None),
            Outcome::Forward(_) => RequestOutcome::success(None),
        }
    }
}

