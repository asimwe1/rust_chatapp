use std::fmt::Debug;

use request::Request;
use http::{ContentType, Method, Cookies};

pub trait FromRequest<'r>: Sized {
    type Error: Debug;

    fn from_request(request: &'r Request) -> Result<Self, Self::Error>;
}

impl<'r> FromRequest<'r> for &'r Request {
    type Error = ();

    fn from_request(request: &'r Request) -> Result<Self, Self::Error> {
        Ok(request)
    }
}

impl<'r> FromRequest<'r> for Method {
    type Error = ();

    fn from_request(request: &'r Request) -> Result<Self, Self::Error> {
        Ok(request.method)
    }
}

impl<'r> FromRequest<'r> for &'r Cookies {
    type Error = ();
    fn from_request(request: &'r Request) -> Result<Self, Self::Error> {
        Ok(request.cookies())
    }
}

impl<'r> FromRequest<'r> for ContentType {
    type Error = ();

    fn from_request(request: &'r Request) -> Result<Self, Self::Error> {
        Ok(request.content_type())
    }
}

impl<'r, T: FromRequest<'r>> FromRequest<'r> for Option<T> {
    type Error = ();

    fn from_request(request: &'r Request) -> Result<Self, Self::Error> {
        let opt = match T::from_request(request) {
            Ok(v) => Some(v),
            Err(_) => None,
        };

        Ok(opt)
    }
}

impl<'r, T: FromRequest<'r>> FromRequest<'r> for Result<T, T::Error> {
    type Error = ();

    fn from_request(request: &'r Request) -> Result<Self, Self::Error> {
        Ok(T::from_request(request))
    }
}
