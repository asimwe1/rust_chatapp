use request::Request;
use response::Response;
use method::Method;

use std::fmt;
use term_painter::Color::*;
use term_painter::ToStyle;

pub type Handler<'a> = fn(Request) -> Response<'a>;

// TODO: Figure out if using 'static for Handler is a good idea.
// TODO: Merge this `Route` and route::Route, somewhow.
pub struct Route {
    pub method: Method,
    pub path: &'static str,
    pub handler: Handler<'static>
}

impl Route {
    pub fn new(method: Method, path: &'static str, handler: Handler<'static>)
            -> Route {
        Route {
            method: method,
            path: path,
            handler: handler
        }
    }
}

impl<'a> fmt::Display for Route {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {:?}", Green.paint(&self.method), Blue.paint(&self.path))
    }
}

