use request::*;
use method::Method;
use std::fmt::Debug;
use content_type::ContentType;

pub trait FromRequest<'r, 'c>: Sized {
    type Error: Debug;

    fn from_request(request: &'r Request<'c>) -> Result<Self, Self::Error>;
}

impl<'r, 'c> FromRequest<'r, 'c> for &'r Request<'c> {
    type Error = ();

    fn from_request(request: &'r Request<'c>) -> Result<Self, Self::Error> {
        Ok(request)
    }
}

impl<'r, 'c> FromRequest<'r, 'c> for Method {
    type Error = ();

    fn from_request(request: &'r Request<'c>) -> Result<Self, Self::Error> {
        Ok(request.method)
    }
}

impl<'r, 'c> FromRequest<'r, 'c> for Cookies {
    type Error = ();

    fn from_request(request: &'r Request<'c>) -> Result<Self, Self::Error> {
        match request.headers().get::<HyperCookie>() {
           // TODO: What to do about key?
           Some(cookie) => Ok(cookie.to_cookie_jar(&[])),
           None => Ok(Cookies::new(&[]))
        }
    }
}

impl<'r, 'c> FromRequest<'r, 'c> for ContentType {
    type Error = ();

    fn from_request(request: &'r Request<'c>) -> Result<Self, Self::Error> {
        Ok(request.content_type())
    }
}

impl<'r, 'c, T: FromRequest<'r, 'c>> FromRequest<'r, 'c> for Option<T> {
    type Error = ();

    fn from_request(request: &'r Request<'c>) -> Result<Self, Self::Error> {
        let opt = match T::from_request(request) {
            Ok(v) => Some(v),
            Err(_) => None
        };

        Ok(opt)
    }
}

impl<'r, 'c, T: FromRequest<'r, 'c>> FromRequest<'r, 'c> for Result<T, T::Error> {
    type Error = ();

    fn from_request(request: &'r Request<'c>) -> Result<Self, Self::Error> {
        Ok(T::from_request(request))
    }
}
