use std::fmt;
use std::borrow::Cow;

use crate::http::uri::{self, Origin};
use crate::http::ext::IntoOwned;
use crate::form::ValueField;
use crate::router::segment::Segment;

/// A URI or a route, used to match against requests.
///
/// A route URI is composed of two components:
///
///   * `base`
///
///     Otherwise known as the route's "mount point", the `base` is a static
///     [`Origin`] that prefixes the route URI. All route URIs have a `base`.
///     When routes are created manually with [`Route::new()`], the base
///     defaults to `/`. When mounted via [`Rocket::mount()`], the base is
///     explicitly specified as the first argument.
///
///     ```rust
///     use rocket::Route;
///     use rocket::http::Method;
///     # use rocket::handler::dummy as handler;
///
///     let route = Route::new(Method::Get, "/foo/<bar>", handler);
///     assert_eq!(route.uri.base(), "/");
///
///     let rocket = rocket::ignite().mount("/base", vec![route]);
///     let routes: Vec<_> = rocket.routes().collect();
///     assert_eq!(routes[0].uri.base(), "/base");
///     ```
///
///   * `origin`
///
///     Otherwise known as the "route URI", the `origin` is an [`Origin`] with
///     potentially dynamic (`<dyn>` or `<dyn..>`) segments. It is prefixed with
///     the `base`. This is the URI which is matched against incoming requests
///     for routing.
///
///     ```rust
///     use rocket::Route;
///     use rocket::http::Method;
///     # use rocket::handler::dummy as handler;
///
///     let route = Route::new(Method::Get, "/foo/<bar>", handler);
///     assert_eq!(route.uri, "/foo/<bar>");
///
///     let rocket = rocket::ignite().mount("/base", vec![route]);
///     let routes: Vec<_> = rocket.routes().collect();
///     assert_eq!(routes[0].uri, "/base/foo/<bar>");
///     ```
///
/// [`Rocket::mount()`]: crate::Rocket::mount()
/// [`Route::new()`]: crate::Route::new()
#[derive(Clone)]
pub struct RouteUri<'a> {
    /// The source string for this URI.
    source: Cow<'a, str>,
    /// The mount point.
    pub base: Origin<'a>,
    /// The URI _without_ the `base` mount point.
    pub unmounted_origin: Origin<'a>,
    /// The URI _with_ the base mount point. This is the canoncical route URI.
    pub origin: Origin<'a>,
    /// Cached metadata about this URI.
    pub(crate) metadata: Metadata,
}

#[derive(Debug, Default, Clone)]
pub(crate) struct Metadata {
    /// Segments in the base.
    pub base_segs: Vec<Segment>,
    /// Segments in the path, including base.
    pub path_segs: Vec<Segment>,
    /// Segments in the query.
    pub query_segs: Vec<Segment>,
    /// `(name, value)` of the query segments that are static.
    pub static_query_fields: Vec<(String, String)>,
    /// Whether the path is completely static.
    pub static_path: bool,
    /// Whether the path is completely dynamic.
    pub wild_path: bool,
    /// Whether the path has a `<trailing..>` parameter.
    pub trailing_path: bool,
    /// Whether the query is completely dynamic.
    pub wild_query: bool,
}

type Result<T> = std::result::Result<T, uri::Error<'static>>;

impl<'a> RouteUri<'a> {
    /// Create a new `RouteUri`.
    ///
    /// This is a fallible variant of [`RouteUri::new`] which returns an `Err`
    /// if `base` or `uri` cannot be parsed as [`Origin`]s.
    pub(crate) fn try_new(base: &str, uri: &str) -> Result<RouteUri<'static>> {
        let mut base = Origin::parse(base)
            .map_err(|e| e.into_owned())?
            .into_normalized()
            .into_owned();

        base.clear_query();

        let unmounted_origin = Origin::parse_route(uri)
            .map_err(|e| e.into_owned())?
            .into_normalized()
            .into_owned();

        let origin = Origin::parse_route(&format!("{}/{}", base, unmounted_origin))
            .map_err(|e| e.into_owned())?
            .into_normalized()
            .into_owned();

        let source = origin.to_string().into();
        let metadata = Metadata::from(&base, &origin);

        Ok(RouteUri { source, unmounted_origin, base, origin, metadata })
    }

    /// Create a new `RouteUri`.
    ///
    /// Panics if  `base` or `uri` cannot be parsed as `Origin`s.
    pub(crate) fn new(base: &str, uri: &str) -> RouteUri<'static> {
        Self::try_new(base, uri).expect("Expected valid URIs")
    }

    /// The path of the base mount point of this route URI as an `&str`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket::Route;
    /// use rocket::http::Method;
    /// # use rocket::handler::{dummy as handler, Outcome, HandlerFuture};
    ///
    /// let index = Route::new(Method::Get, "/foo/bar?a=1", handler);
    /// assert_eq!(index.uri.base(), "/");
    /// let index = index.map_base(|base| format!("{}{}", "/boo", base)).unwrap();
    /// assert_eq!(index.uri.base(), "/boo");
    /// ```
    #[inline(always)]
    pub fn base(&self) -> &str {
        self.base.path().as_str()
    }

    /// The path part of this route URI as an `&str`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket::Route;
    /// use rocket::http::Method;
    /// # use rocket::handler::{dummy as handler, Outcome, HandlerFuture};
    ///
    /// let index = Route::new(Method::Get, "/foo/bar?a=1", handler);
    /// assert_eq!(index.uri.path(), "/foo/bar");
    /// let index = index.map_base(|base| format!("{}{}", "/boo", base)).unwrap();
    /// assert_eq!(index.uri.path(), "/boo/foo/bar");
    /// ```
    #[inline(always)]
    pub fn path(&self) -> &str {
        self.origin.path().as_str()
    }

    /// The query part of this route URI as an `Option<&str>`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket::Route;
    /// use rocket::http::Method;
    /// # use rocket::handler::{dummy as handler, Outcome, HandlerFuture};
    ///
    /// let index = Route::new(Method::Get, "/foo/bar?a=1", handler);
    /// assert_eq!(index.uri.query(), Some("a=1"));
    /// let index = index.map_base(|base| format!("{}{}", "/boo", base)).unwrap();
    /// assert_eq!(index.uri.query(), Some("a=1"));
    /// ```
    #[inline(always)]
    pub fn query(&self) -> Option<&str> {
        self.origin.query().map(|q| q.as_str())
    }

    /// The full URI as an `&str`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket::Route;
    /// use rocket::http::Method;
    /// # use rocket::handler::{dummy as handler, Outcome, HandlerFuture};
    ///
    /// let index = Route::new(Method::Get, "/foo/bar?a=1", handler);
    /// assert_eq!(index.uri.as_str(), "/foo/bar?a=1");
    /// let index = index.map_base(|base| format!("{}{}", "/boo", base)).unwrap();
    /// assert_eq!(index.uri.as_str(), "/boo/foo/bar?a=1");
    /// ```
    #[inline(always)]
    pub fn as_str(&self) -> &str {
        &self.source
    }

    /// Get the default rank of a route with this URI.
    ///
    /// The route's default rank is determined based on the presence or absence
    /// of static and dynamic paths and queries. See the documentation for
    /// [`Route::new`][`crate::Route::new`] for a table summarizing the exact default ranks.
    pub(crate) fn default_rank(&self) -> isize {
        let static_path = self.metadata.static_path;
        let wild_query = self.query().map(|_| self.metadata.wild_query);
        match (static_path, wild_query) {
            (true, Some(false)) => -6,   // static path, partly static query
            (true, Some(true)) => -5,    // static path, fully dynamic query
            (true, None) => -4,          // static path, no query
            (false, Some(false)) => -3,  // dynamic path, partly static query
            (false, Some(true)) => -2,   // dynamic path, fully dynamic query
            (false, None) => -1,         // dynamic path, no query
        }
    }
}

impl Metadata {
    fn from(base: &Origin<'_>, origin: &Origin<'_>) -> Self {
        let base_segs = base.raw_path_segments()
            .map(Segment::from)
            .collect::<Vec<_>>();

        let path_segs = origin.raw_path_segments()
            .map(Segment::from)
            .collect::<Vec<_>>();

        let query_segs = origin.raw_query_segments()
            .map(Segment::from)
            .collect::<Vec<_>>();

        Metadata {
            static_path: path_segs.iter().all(|s| !s.dynamic),
            wild_path: path_segs.iter().all(|s| s.dynamic)
                && path_segs.last().map_or(false, |p| p.trailing),
            trailing_path: path_segs.last().map_or(false, |p| p.trailing),
            wild_query: query_segs.iter().all(|s| s.dynamic),
            static_query_fields: query_segs.iter().filter(|s| !s.dynamic)
                .map(|s| ValueField::parse(&s.value))
                .map(|f| (f.name.source().to_string(), f.value.to_string()))
                .collect(),
            path_segs,
            query_segs,
            base_segs,
        }
    }
}

impl<'a> std::ops::Deref for RouteUri<'a> {
    type Target = Origin<'a>;

    fn deref(&self) -> &Self::Target {
        &self.origin
    }
}

impl fmt::Display for RouteUri<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.origin.fmt(f)
    }
}

impl fmt::Debug for RouteUri<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RouteUri")
            .field("base", &self.base)
            .field("unmounted_origin", &self.unmounted_origin)
            .field("origin", &self.origin)
            .finish()
    }
}

impl<'a, 'b> PartialEq<Origin<'b>> for RouteUri<'a> {
    fn eq(&self, other: &Origin<'b>) -> bool { &self.origin == other }
}

impl PartialEq<str> for RouteUri<'_> {
    fn eq(&self, other: &str) -> bool { self.as_str() == other }
}

impl PartialEq<&str> for RouteUri<'_> {
    fn eq(&self, other: &&str) -> bool { self.as_str() == *other }
}
