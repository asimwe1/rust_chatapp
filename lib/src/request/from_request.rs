use std::fmt::Debug;

use outcome::{self, IntoOutcome};
use request::Request;
use outcome::Outcome::*;
use http::{StatusCode, ContentType, Method, Cookies};

/// Type alias for the `Outcome` of a `FromRequest` conversion.
pub type Outcome<T, E> = outcome::Outcome<T, (StatusCode, E), ()>;

impl<T, E> IntoOutcome<T, (StatusCode, E), ()> for Result<T, E> {
    fn into_outcome(self) -> Outcome<T, E> {
        match self {
            Ok(val) => Success(val),
            Err(val) => Failure((StatusCode::BadRequest, val))
        }
    }
}

pub trait FromRequest<'r>: Sized {
    type Error: Debug;

    fn from_request(request: &'r Request) -> Outcome<Self, Self::Error>;
}

impl<'r> FromRequest<'r> for &'r Request {
    type Error = ();

    fn from_request(request: &'r Request) -> Outcome<Self, Self::Error> {
        Success(request)
    }
}

impl<'r> FromRequest<'r> for Method {
    type Error = ();

    fn from_request(request: &'r Request) -> Outcome<Self, Self::Error> {
        Success(request.method)
    }
}

impl<'r> FromRequest<'r> for &'r Cookies {
    type Error = ();

    fn from_request(request: &'r Request) -> Outcome<Self, Self::Error> {
        Success(request.cookies())
    }
}

impl<'r> FromRequest<'r> for ContentType {
    type Error = ();

    fn from_request(request: &'r Request) -> Outcome<Self, Self::Error> {
        Success(request.content_type())
    }
}

impl<'r, T: FromRequest<'r>> FromRequest<'r> for Result<T, T::Error> {
    type Error = ();

    fn from_request(request: &'r Request) -> Outcome<Self, Self::Error> {
        match T::from_request(request) {
            Success(val) => Success(Ok(val)),
            Failure((_, e)) => Success(Err(e)),
            Forward(_) => Forward(()),
        }
    }
}

impl<'r, T: FromRequest<'r>> FromRequest<'r> for Option<T> {
    type Error = ();

    fn from_request(request: &'r Request) -> Outcome<Self, Self::Error> {
        match T::from_request(request) {
            Success(val) => Success(Some(val)),
            Failure(_) => Success(None),
            Forward(_) => Success(None),
        }
    }
}

