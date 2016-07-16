use request::Request;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Error {
    BadMethod,
    BadParse,
    NoRoute,
    NoKey
}

pub struct RoutingError<'r> {
    pub error: Error,
    pub request: Request<'r>,
    pub chain: Option<&'r [&'r str]>
}

impl<'a> RoutingError<'a> {
    pub fn unchained(request: Request<'a>)
            -> RoutingError<'a> {
        RoutingError {
            error: Error::NoRoute,
            request: request,
            chain: None,
        }
    }

    pub fn new(error: Error, request: Request<'a>, chain: &'a [&'a str])
        -> RoutingError<'a> {
            RoutingError {
                error: error,
                request: request,
                chain: Some(chain)
            }
    }
}
