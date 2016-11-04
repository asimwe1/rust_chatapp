use std::fmt;
use std::convert::From;

use super::Collider; // :D

use term_painter::ToStyle;
use term_painter::Color::*;

use codegen::StaticRouteInfo;
use handler::Handler;
use request::Request;
use http::{Method, ContentType};
use http::uri::{URI, URIBuf};

/// A route: a method, its handler, path, rank, and format/content type.
pub struct Route {
    /// The method this route matches against.
    pub method: Method,
    /// A function that should be called when the route matches.
    pub handler: Handler,
    /// The path (in Rocket format) that should be matched against.
    pub path: URIBuf,
    /// The rank of this route. Lower ranks have higher priorities.
    pub rank: isize,
    /// The Content-Type this route matches against.
    pub content_type: ContentType,
}

fn default_rank(path: &str) -> isize {
    // The rank for a given path is 0 if it is a static route (it doesn't
    // contain any dynamic <segmants>) or 1 if it is dynamic.
    path.contains('<') as isize
}

impl Route {
    /// Creates a new route with the method, path, and handler.
    ///
    /// The rank of the route will be `0` if the path contains no dynamic
    /// segments, and `1` if it does.
    pub fn new<S>(m: Method, path: S, handler: Handler) -> Route
        where S: AsRef<str>
    {
        Route {
            method: m,
            handler: handler,
            rank: default_rank(path.as_ref()),
            path: URIBuf::from(path.as_ref()),
            content_type: ContentType::any(),
        }
    }

    /// Creates a new route with the given rank, method, path, and handler.
    pub fn ranked<S>(rank: isize, m: Method, path: S, handler: Handler) -> Route
        where S: AsRef<str>
    {
        Route {
            method: m,
            path: URIBuf::from(path.as_ref()),
            handler: handler,
            rank: rank,
            content_type: ContentType::any(),
        }
    }

    /// Sets the path of the route. Does not update the rank or any other
    /// parameters.
    pub fn set_path<S>(&mut self, path: S)
        where S: AsRef<str>
    {
        self.path = URIBuf::from(path.as_ref());
    }

    // FIXME: Decide whether a component has to be fully variable or not. That
    // is, whether you can have: /a<a>b/ or even /<a>:<b>/
    // TODO: Don't return a Vec...take in an &mut [&'a str] (no alloc!)
    /// Given a URI, returns a vector of slices of that URI corresponding to the
    /// dynamic segments in this route.
    #[doc(hidden)]
    pub fn get_params<'a>(&self, uri: URI<'a>) -> Vec<&'a str> {
        let route_segs = self.path.as_uri().segments();
        let uri_segs = uri.segments();

        let mut result = Vec::with_capacity(self.path.segment_count());
        for (route_seg, uri_seg) in route_segs.zip(uri_segs) {
            if route_seg.ends_with("..>") {
                // FIXME: Here.
                break;
            } else if route_seg.ends_with('>') {
                // FIXME: Here.
                result.push(uri_seg);
            }
        }

        result
    }
}

impl Clone for Route {
    fn clone(&self) -> Route {
        Route {
            method: self.method,
            handler: self.handler,
            rank: self.rank,
            path: self.path.clone(),
            content_type: self.content_type.clone(),
        }
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

impl fmt::Debug for Route {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        <Route as fmt::Display>::fmt(self, f)
    }
}

#[doc(hidden)]
impl<'a> From<&'a StaticRouteInfo> for Route {
    fn from(info: &'a StaticRouteInfo) -> Route {
        let mut route = Route::new(info.method, info.path, info.handler);
        route.content_type = info.format.clone().unwrap_or(ContentType::any());
        if let Some(rank) = info.rank {
            route.rank = rank;
        }

        route
    }
}

impl Collider for Route {
    fn collides_with(&self, b: &Route) -> bool {
        self.method == b.method
            && self.rank == b.rank
            && self.content_type.collides_with(&b.content_type)
            && self.path.as_uri().collides_with(&b.path.as_uri())
    }
}

impl Collider<Request> for Route {
    fn collides_with(&self, req: &Request) -> bool {
        self.method == req.method
            && req.uri().collides_with(&self.path.as_uri())
            && req.content_type().collides_with(&self.content_type)
    }
}
