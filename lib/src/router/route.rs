use ::{Method, Handler, StaticRouteInfo};
use content_type::ContentType;
use super::{Collider, URI, URIBuf}; // :D

use term_painter::ToStyle;
use term_painter::Color::*;

use std::fmt;
use std::convert::From;
use request::Request;

pub struct Route {
    pub method: Method,
    pub handler: Handler,
    pub path: URIBuf,
    pub rank: isize, // Lower ranks have higher priorities.
    pub content_type: ContentType,
}

fn default_rank(path: &str) -> isize {
    // The rank for a given path is 0 if it is a static route (it doesn't
    // contain any dynamic <segmants>) or 1 if it is dynamic.
    path.contains('<') as isize
}

impl Route {
    pub fn ranked<S>(rank: isize, m: Method, path: S, handler: Handler)
            -> Route where S: AsRef<str> {
        Route {
            method: m,
            path: URIBuf::from(path.as_ref()),
            handler: handler,
            rank: rank,
            content_type: ContentType::any(),
        }
    }

    pub fn new<S>(m: Method, path: S, handler: Handler) -> Route where S: AsRef<str> {
        Route {
            method: m,
            handler: handler,
            rank: default_rank(path.as_ref()),
            path: URIBuf::from(path.as_ref()),
            content_type: ContentType::any(),
        }
    }

    pub fn set_path<S>(&mut self, path: S) where S: AsRef<str> {
        self.path = URIBuf::from(path.as_ref());
    }

    // FIXME: Decide whether a component has to be fully variable or not. That
    // is, whether you can have: /a<a>b/ or even /<a>:<b>/
    // TODO: Don't return a Vec...take in an &mut [&'a str] (no alloc!)
    pub fn get_params<'a>(&self, uri: URI<'a>) -> Vec<&'a str> {
        let route_components = self.path.segments();
        let uri_components = uri.segments();

        let mut result = Vec::with_capacity(self.path.segment_count());
        for (route_seg, uri_seg) in route_components.zip(uri_components) {
            if route_seg.starts_with('<') { // FIXME: Here.
                result.push(uri_seg);
            }
        }

        result
    }
}

impl fmt::Display for Route {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", Green.paint(&self.method), Blue.paint(&self.path))?;

        if self.rank > 1 {
            write!(f, " [{}]", White.paint(&self.rank))?;
        }

        if !self.content_type.is_any() {
            write!(f, " {}", Yellow.paint(&self.content_type))
        } else {
            Ok(())
        }
    }
}

impl<'a> From<&'a StaticRouteInfo> for Route {
    fn from(info: &'a StaticRouteInfo) -> Route {
        let mut route = Route::new(info.method, info.path, info.handler);
        route.content_type = info.content_type.clone();
        if let Some(rank) = info.rank {
            route.rank = rank;
        }

        route
    }
}

impl Collider for Route {
    fn collides_with(&self, b: &Route) -> bool {
        self.path.segment_count() == b.path.segment_count()
            && self.method == b.method
            && self.rank == b.rank
            && self.content_type.collides_with(&b.content_type)
            && self.path.collides_with(&b.path)
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

impl<'r> Collider<Request<'r>> for Route {
    fn collides_with(&self, req: &Request) -> bool {
        self.method == req.method
            && req.uri.collides_with(&self.path)
            && req.content_type().collides_with(&self.content_type)
    }
}
