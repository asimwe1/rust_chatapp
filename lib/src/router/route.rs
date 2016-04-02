use term_painter::ToStyle;
use term_painter::Color::*;
use std::fmt;
use method::Method;
use super::{Collider, URI, URIBuf}; // :D
use handler::Handler;

// TODO: Add ranking to routes. Give static routes higher rank by default.
// FIXME: Take in the handler! Or maybe keep that in `Router`?
pub struct Route {
    pub method: Method,
    pub handler: Handler<'static>,
    pub path: URIBuf,
    pub rank: isize
}

impl Route {
    pub fn ranked(rank: isize, m: Method, path: String,
                  handler: Handler<'static>) -> Route {
        Route {
            method: m,
            path: URIBuf::new(path),
            handler: handler,
            rank: rank
        }
    }

    pub fn new(m: Method, path: String, handler: Handler<'static>) -> Route {
        Route {
            method: m,
            handler: handler,
            rank: (!path.contains("<") as isize),
            path: URIBuf::new(path),
        }
    }

    pub fn set_path(&mut self, path: String) {
        self.path = URIBuf::new(path);
    }

    // FIXME: Decide whether a component has to be fully variable or not. That
    // is, whether you can have: /a<a>b/ or even /<a>:<b>/
    // TODO: Don't return a Vec...take in an &mut [&'a str] (no alloc!)
    pub fn get_params<'a>(&self, uri: &'a str) -> Vec<&'a str> {
        let route_components = self.path.segments();
        let uri_components = URI::new(uri).segments();

        let mut result = Vec::with_capacity(self.path.segment_count());
        for (route_seg, uri_seg) in route_components.zip(uri_components) {
            if route_seg.starts_with("<") { // FIXME: Here.
                result.push(uri_seg);
            }
        }

        result
    }
}

impl fmt::Display for Route {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {:?}", Green.paint(&self.method), Blue.paint(&self.path))
    }
}

impl Collider for Route {
    fn collides_with(&self, b: &Route) -> bool {
        if self.path.segment_count() != b.path.segment_count()
                || self.method != b.method
                || self.rank != b.rank {
            return false;
        }

        self.path.collides_with(&b.path)
    }
}

impl<'a> Collider<Route> for &'a str {
    fn collides_with(&self, other: &Route) -> bool {
        let path = URI::new(self);
        path.collides_with(&other.path)
    }
}

impl Collider<str> for Route {
    fn collides_with(&self, other: &str) -> bool {
        other.collides_with(self)
    }
}
