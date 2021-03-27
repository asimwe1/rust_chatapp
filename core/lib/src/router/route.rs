use std::fmt;
use std::convert::From;
use std::borrow::Cow;

use yansi::Paint;

use crate::codegen::StaticRouteInfo;
use crate::handler::Handler;
use crate::http::{uri, Method, MediaType};
use crate::router::RouteUri;

/// A route: a method, its handler, path, rank, and format/media type.
#[derive(Clone)]
pub struct Route {
    /// The name of this route, if one was given.
    pub name: Option<Cow<'static, str>>,
    /// The method this route matches against.
    pub method: Method,
    /// The function that should be called when the route matches.
    pub handler: Box<dyn Handler>,
    /// The route URI.
    pub uri: RouteUri<'static>,
    /// The rank of this route. Lower ranks have higher priorities.
    pub rank: isize,
    /// The media type this route matches against, if any.
    pub format: Option<MediaType>,
}

impl Route {
    /// Creates a new route with the given method, path, and handler with a base
    /// of `/`.
    ///
    /// # Ranking
    ///
    /// The default rank prefers static components over dynamic components in
    /// both paths and queries: the _more_ static a route's path and query are,
    /// the higher its precedence.
    ///
    /// There are three "colors" to paths and queries:
    ///   1. `static`, meaning all components are static
    ///   2. `partial`, meaning at least one component is dynamic
    ///   3. `wild`, meaning all components are dynamic
    ///
    /// Static paths carry more weight than static queries. The same is true for
    /// partial and wild paths. This results in the following default ranking
    /// table:
    ///
    /// | path    | query   | rank |
    /// |---------|---------|------|
    /// | static  | static  | -12  |
    /// | static  | partial | -11  |
    /// | static  | wild    | -10  |
    /// | static  | none    | -9   |
    /// | partial | static  | -8   |
    /// | partial | partial | -7   |
    /// | partial | wild    | -6   |
    /// | partial | none    | -5   |
    /// | wild    | static  | -4   |
    /// | wild    | partial | -3   |
    /// | wild    | wild    | -2   |
    /// | wild    | none    | -1   |
    ///
    /// Note that _lower_ ranks have _higher_ precedence.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket::Route;
    /// use rocket::http::Method;
    /// # use rocket::handler::{dummy as handler};
    ///
    /// macro_rules! assert_rank {
    ///     ($($uri:expr => $rank:expr,)*) => {$(
    ///         let route = Route::new(Method::Get, $uri, handler);
    ///         assert_eq!(route.rank, $rank);
    ///     )*}
    /// }
    ///
    /// assert_rank! {
    ///     "/?foo" => -12,                 // static path, static query
    ///     "/foo/bar?a=b&bob" => -12,      // static path, static query
    ///     "/?a=b&bob" => -12,             // static path, static query
    ///
    ///     "/?a&<zoo..>" => -11,           // static path, partial query
    ///     "/foo?a&<zoo..>" => -11,        // static path, partial query
    ///     "/?a&<zoo>" => -11,             // static path, partial query
    ///
    ///     "/?<zoo..>" => -10,             // static path, wild query
    ///     "/foo?<zoo..>" => -10,          // static path, wild query
    ///     "/foo?<a>&<b>" => -10,          // static path, wild query
    ///
    ///     "/" => -9,                      // static path, no query
    ///     "/foo/bar" => -9,               // static path, no query
    ///
    ///     "/a/<b>?foo" => -8,             // partial path, static query
    ///     "/a/<b..>?foo" => -8,           // partial path, static query
    ///     "/<a>/b?foo" => -8,             // partial path, static query
    ///
    ///     "/a/<b>?<b>&c" => -7,           // partial path, partial query
    ///     "/a/<b..>?a&<c..>" => -7,       // partial path, partial query
    ///
    ///     "/a/<b>?<c..>" => -6,           // partial path, wild query
    ///     "/a/<b..>?<c>&<d>" => -6,       // partial path, wild query
    ///     "/a/<b..>?<c>" => -6,           // partial path, wild query
    ///
    ///     "/a/<b>" => -5,                 // partial path, no query
    ///     "/<a>/b" => -5,                 // partial path, no query
    ///     "/a/<b..>" => -5,               // partial path, no query
    ///
    ///     "/<b>/<c>?foo&bar" => -4,       // wild path, static query
    ///     "/<a>/<b..>?foo" => -4,         // wild path, static query
    ///     "/<b..>?cat" => -4,             // wild path, static query
    ///
    ///     "/<b>/<c>?<foo>&bar" => -3,     // wild path, partial query
    ///     "/<a>/<b..>?a&<b..>" => -3,     // wild path, partial query
    ///     "/<b..>?cat&<dog>" => -3,       // wild path, partial query
    ///
    ///     "/<b>/<c>?<foo>" => -2,         // wild path, wild query
    ///     "/<a>/<b..>?<b..>" => -2,       // wild path, wild query
    ///     "/<b..>?<c>&<dog>" => -2,       // wild path, wild query
    ///
    ///     "/<b>/<c>" => -1,               // wild path, no query
    ///     "/<a>/<b..>" => -1,             // wild path, no query
    ///     "/<b..>" => -1,                 // wild path, no query
    /// }
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if `path` is not a valid origin URI or Rocket route URI.
    pub fn new<H: Handler>(method: Method, uri: &str, handler: H) -> Route {
        Route::ranked(None, method, uri, handler)
    }

    /// Creates a new route with the given rank, method, path, and handler with
    /// a base of `/`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket::Route;
    /// use rocket::http::Method;
    /// # use rocket::handler::{dummy as handler};
    ///
    /// // this is a rank 1 route matching requests to `GET /`
    /// let index = Route::ranked(1, Method::Get, "/", handler);
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if `path` is not a valid origin URI or Rocket route URI.
    pub fn ranked<H, R>(rank: R, method: Method, uri: &str, handler: H) -> Route
        where H: Handler + 'static, R: Into<Option<isize>>,
    {
        let uri = RouteUri::new("/", uri);
        let rank = rank.into().unwrap_or_else(|| uri.default_rank());
        Route {
            name: None,
            format: None,
            handler: Box::new(handler),
            rank, uri, method,
        }
    }


    /// Maps the `base` of this route using `mapper`, returning a new `Route`
    /// with the returned base.
    ///
    /// `mapper` is called with the current base. The returned `String` is used
    /// as the new base if it is a valid URI. If the returned base URI contains
    /// a query, it is ignored. Returns an error if the base produced by
    /// `mapper` is not a valid origin URI.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket::Route;
    /// use rocket::http::{Method, uri::Origin};
    /// # use rocket::handler::{dummy as handler, Outcome, HandlerFuture};
    ///
    /// let index = Route::new(Method::Get, "/foo/bar", handler);
    /// assert_eq!(index.uri.base(), "/");
    /// assert_eq!(index.uri.unmounted_origin.path(), "/foo/bar");
    /// assert_eq!(index.uri.path(), "/foo/bar");
    ///
    /// let index = index.map_base(|base| format!("{}{}", "/boo", base)).unwrap();
    /// assert_eq!(index.uri.base(), "/boo");
    /// assert_eq!(index.uri.unmounted_origin.path(), "/foo/bar");
    /// assert_eq!(index.uri.path(), "/boo/foo/bar");
    /// ```
    pub fn map_base<'a, F>(mut self, mapper: F) -> Result<Self, uri::Error<'static>>
        where F: FnOnce(uri::Origin<'a>) -> String
    {
        let base = mapper(self.uri.base);
        self.uri = RouteUri::try_new(&base, &self.uri.unmounted_origin.to_string())?;
        Ok(self)
    }
}

impl fmt::Display for Route {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref n) = self.name {
            write!(f, "{}{}{} ", Paint::cyan("("), Paint::white(n), Paint::cyan(")"))?;
        }

        write!(f, "{} ", Paint::green(&self.method))?;
        if self.uri.base() != "/" {
            write!(f, "{}", Paint::blue(self.uri.base()).underline())?;
        }

        write!(f, "{}", Paint::blue(&self.uri.unmounted_origin))?;

        if self.rank > 1 {
            write!(f, " [{}]", Paint::default(&self.rank).bold())?;
        }

        if let Some(ref format) = self.format {
            write!(f, " {}", Paint::yellow(format))?;
        }

        Ok(())
    }
}

impl fmt::Debug for Route {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Route")
            .field("name", &self.name)
            .field("method", &self.method)
            .field("uri", &self.uri)
            .field("rank", &self.rank)
            .field("format", &self.format)
            .finish()
    }
}

#[doc(hidden)]
impl From<StaticRouteInfo> for Route {
    fn from(info: StaticRouteInfo) -> Route {
        // This should never panic since `info.path` is statically checked.
        let mut route = Route::new(info.method, info.path, info.handler);
        route.format = info.format;
        route.name = Some(info.name.into());
        if let Some(rank) = info.rank {
            route.rank = rank;
        }

        route
    }
}
