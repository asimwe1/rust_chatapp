use crate::catcher::Catcher;
use crate::route::{Route, Segment, RouteUri};

use crate::http::MediaType;

pub trait Collide<T = Self> {
    fn collides_with(&self, other: &T) -> bool;
}

impl Collide for Route {
    /// Determines if two routes can match against some request. That is, if two
    /// routes `collide`, there exists a request that can match against both
    /// routes.
    ///
    /// This implementation is used at initialization to check if two user
    /// routes collide before launching. Format collisions works like this:
    ///
    ///   * If route specifies a format, it only gets requests for that format.
    ///   * If route doesn't specify a format, it gets requests for any format.
    ///
    /// Because query parsing is lenient, and dynamic query parameters can be
    /// missing, the particularities of a query string do not impact whether two
    /// routes collide. The query effects the route's color, however, which
    /// effects its rank.
    fn collides_with(&self, other: &Route) -> bool {
        self.method == other.method
            && self.rank == other.rank
            && self.uri.collides_with(&other.uri)
            && formats_collide(self, other)
    }
}

impl Collide for Catcher {
    /// Determines if two catchers are in conflict: there exists a request for
    /// which there exist no rule to determine _which_ of the two catchers to
    /// use. This means that the catchers:
    ///
    ///  * Have the same base.
    ///  * Have the same status code or are both defaults.
    fn collides_with(&self, other: &Self) -> bool {
        self.code == other.code
            && self.base.path().segments().eq(other.base.path().segments())
    }
}

impl Collide for RouteUri<'_> {
    fn collides_with(&self, other: &Self) -> bool {
        let a_segments = &self.metadata.uri_segments;
        let b_segments = &other.metadata.uri_segments;
        for (seg_a, seg_b) in a_segments.iter().zip(b_segments.iter()) {
            if seg_a.dynamic_trail || seg_b.dynamic_trail {
                return true;
            }

            if !seg_a.collides_with(seg_b) {
                return false;
            }
        }

        a_segments.len() == b_segments.len()
    }
}

impl Collide for Segment {
    fn collides_with(&self, other: &Self) -> bool {
        self.dynamic || other.dynamic || self.value == other.value
    }
}

impl Collide for MediaType {
    fn collides_with(&self, other: &Self) -> bool {
        let collide = |a, b| a == "*" || b == "*" || a == b;
        collide(self.top(), other.top()) && collide(self.sub(), other.sub())
    }
}

fn formats_collide(route: &Route, other: &Route) -> bool {
    // When matching against the `Accept` header, the client can always provide
    // a media type that will cause a collision through non-specificity, i.e,
    // `*/*` matches everything.
    if !route.method.supports_payload() {
        return true;
    }

    // When matching against the `Content-Type` header, we'll only consider
    // requests as having a `Content-Type` if they're fully specified. If a
    // route doesn't have a `format`, it accepts all `Content-Type`s. If a
    // request doesn't have a format, it only matches routes without a format.
    match (route.format.as_ref(), other.format.as_ref()) {
        (Some(a), Some(b)) => a.collides_with(b),
        _ => true
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use crate::route::{Route, dummy_handler};
    use crate::http::{Method, Method::*, MediaType};

    fn dummy_route(ranked: bool, method: impl Into<Option<Method>>, uri: &'static str) -> Route {
        let method = method.into().unwrap_or(Get);
        Route::ranked((!ranked).then(|| 0), method, uri, dummy_handler)
    }

    macro_rules! assert_collision {
        ($ranked:expr, $p1:expr, $p2:expr) => (assert_collision!($ranked, None $p1, None $p2));
        ($ranked:expr, $m1:ident $p1:expr, $m2:ident $p2:expr) => {
            let (a, b) = (dummy_route($ranked, $m1, $p1), dummy_route($ranked, $m2, $p2));
            assert! {
                a.collides_with(&b),
                "\nroutes failed to collide:\n{} does not collide with {}\n", a, b
            }
        };
        (ranked $($t:tt)+) => (assert_collision!(true, $($t)+));
        ($($t:tt)+) => (assert_collision!(false, $($t)+));
    }

    macro_rules! assert_no_collision {
        ($ranked:expr, $p1:expr, $p2:expr) => (assert_no_collision!($ranked, None $p1, None $p2));
        ($ranked:expr, $m1:ident $p1:expr, $m2:ident $p2:expr) => {
            let (a, b) = (dummy_route($ranked, $m1, $p1), dummy_route($ranked, $m2, $p2));
            assert! {
                !a.collides_with(&b),
                "\nunexpected collision:\n{} collides with {}\n", a, b
            }
        };
        (ranked $($t:tt)+) => (assert_no_collision!(true, $($t)+));
        ($($t:tt)+) => (assert_no_collision!(false, $($t)+));
    }

    #[test]
    fn non_collisions() {
        assert_no_collision!("/a", "/b");
        assert_no_collision!("/a/b", "/a");
        assert_no_collision!("/a/b", "/a/c");
        assert_no_collision!("/a/hello", "/a/c");
        assert_no_collision!("/hello", "/a/c");
        assert_no_collision!("/hello/there", "/hello/there/guy");
        assert_no_collision!("/a/<b>", "/b/<b>");
        assert_no_collision!("/<a>/b", "/<b>/a");
        assert_no_collision!("/t", "/test");
        assert_no_collision!("/a", "/aa");
        assert_no_collision!("/a", "/aaa");
        assert_no_collision!("/", "/a");

        assert_no_collision!("/hello", "/hello/");
        assert_no_collision!("/hello/there", "/hello/there/");

        assert_no_collision!("/a?<b>", "/b");
        assert_no_collision!("/a/b", "/a?<b>");
        assert_no_collision!("/a/b/c?<d>", "/a/b/c/d");
        assert_no_collision!("/a/hello", "/a/?<hello>");
        assert_no_collision!("/?<a>", "/hi");

        assert_no_collision!(Get "/", Post "/");
        assert_no_collision!(Post "/", Put "/");
        assert_no_collision!(Put "/a", Put "/");
        assert_no_collision!(Post "/a", Put "/");
        assert_no_collision!(Get "/a", Put "/");
        assert_no_collision!(Get "/hello", Put "/hello");
        assert_no_collision!(Get "/<foo..>", Post "/");

        assert_no_collision!("/a", "/b");
        assert_no_collision!("/a/b", "/a");
        assert_no_collision!("/a/b", "/a/c");
        assert_no_collision!("/a/hello", "/a/c");
        assert_no_collision!("/hello", "/a/c");
        assert_no_collision!("/hello/there", "/hello/there/guy");
        assert_no_collision!("/a/<b>", "/b/<b>");
        assert_no_collision!("/a", "/b");
        assert_no_collision!("/a/b", "/a");
        assert_no_collision!("/a/b", "/a/c");
        assert_no_collision!("/a/hello", "/a/c");
        assert_no_collision!("/hello", "/a/c");
        assert_no_collision!("/hello/there", "/hello/there/guy");
        assert_no_collision!("/a/<b>", "/b/<b>");
        assert_no_collision!("/a", "/b");
        assert_no_collision!("/a/b", "/a");
        assert_no_collision!("/a/b", "/a/c");
        assert_no_collision!("/a/hello", "/a/c");
        assert_no_collision!("/hello", "/a/c");
        assert_no_collision!("/hello/there", "/hello/there/guy");
        assert_no_collision!("/a/<b>", "/b/<b>");
        assert_no_collision!("/t", "/test");
        assert_no_collision!("/a", "/aa");
        assert_no_collision!("/a", "/aaa");
        assert_no_collision!("/", "/a");

        assert_no_collision!("/foo", "/foo/");
        assert_no_collision!("/foo/bar", "/foo/");
        assert_no_collision!("/foo/bar", "/foo/bar/");
        assert_no_collision!("/foo/<a>", "/foo/<a>/");
        assert_no_collision!("/foo/<a>", "/<b>/<a>/");
        assert_no_collision!("/<b>/<a>", "/<b>/<a>/");
        assert_no_collision!("/a/", "/<a>/<b>/<c..>");

        assert_no_collision!("/a", "/a/<a..>");
        assert_no_collision!("/<a>", "/a/<a..>");
        assert_no_collision!("/a/b", "/<a>/<b>/<c..>");
        assert_no_collision!("/a/<b>", "/<a>/<b>/<c..>");
        assert_no_collision!("/<a>/b", "/<a>/<b>/<c..>");
        assert_no_collision!("/hi/<a..>", "/hi");

        assert_no_collision!(ranked "/<a>", "/");
        assert_no_collision!(ranked "/a/", "/<a>/");
        assert_no_collision!(ranked "/hello/<a>", "/hello/");
        assert_no_collision!(ranked "/", "/?a");
        assert_no_collision!(ranked "/", "/?<a>");
        assert_no_collision!(ranked "/a/<b>", "/a/<b>?d");
    }

    #[test]
    fn collisions() {
        assert_collision!("/<a>", "/");
        assert_collision!("/a", "/a");
        assert_collision!("/hello", "/hello");
        assert_collision!("/hello/there/how/ar", "/hello/there/how/ar");
        assert_collision!("/hello/<a>", "/hello/");

        assert_collision!("/<a>", "/<b>");
        assert_collision!("/<a>", "/b");
        assert_collision!("/hello/<name>", "/hello/<person>");
        assert_collision!("/hello/<name>/hi", "/hello/<person>/hi");
        assert_collision!("/hello/<name>/hi/there", "/hello/<person>/hi/there");
        assert_collision!("/<name>/hi/there", "/<person>/hi/there");
        assert_collision!("/<name>/hi/there", "/dude/<name>/there");
        assert_collision!("/<name>/<a>/<b>", "/<a>/<b>/<c>");
        assert_collision!("/<name>/<a>/<b>/", "/<a>/<b>/<c>/");
        assert_collision!("/<a..>", "/hi");
        assert_collision!("/<a..>", "/hi/hey");
        assert_collision!("/<a..>", "/hi/hey/hayo");
        assert_collision!("/a/<a..>", "/a/hi/hey/hayo");
        assert_collision!("/a/<b>/<a..>", "/a/hi/hey/hayo");
        assert_collision!("/a/<b>/<c>/<a..>", "/a/hi/hey/hayo");
        assert_collision!("/<b>/<c>/<a..>", "/a/hi/hey/hayo");
        assert_collision!("/<b>/<c>/hey/hayo", "/a/hi/hey/hayo");
        assert_collision!("/<a..>", "/foo");

        assert_collision!("/", "/<a..>");
        assert_collision!("/a/", "/a/<a..>");
        assert_collision!("/<a>/", "/a/<a..>");
        assert_collision!("/<a>/bar/", "/a/<a..>");

        assert_collision!("/<a>", "/b");
        assert_collision!("/hello/<name>", "/hello/bob");
        assert_collision!("/<name>", "//bob");

        assert_collision!("/<a..>", "///a///");
        assert_collision!("/<a..>", "//a/bcjdklfj//<c>");
        assert_collision!("/a/<a..>", "//a/bcjdklfj//<c>");
        assert_collision!("/a/<b>/<c..>", "//a/bcjdklfj//<c>");
        assert_collision!("/<a..>", "/");
        assert_collision!("/", "/<_..>");
        assert_collision!("/a/b/<a..>", "/a/<b..>");
        assert_collision!("/a/b/<a..>", "/a/<b>/<b..>");
        assert_collision!("/hi/<a..>", "/hi/");
        assert_collision!("/<a..>", "//////");

        assert_collision!("/?<a>", "/?<a>");
        assert_collision!("/a/?<a>", "/a/?<a>");
        assert_collision!("/a?<a>", "/a?<a>");
        assert_collision!("/<r>?<a>", "/<r>?<a>");
        assert_collision!("/a/b/c?<a>", "/a/b/c?<a>");
        assert_collision!("/<a>/b/c?<d>", "/a/b/<c>?<d>");
        assert_collision!("/?<a>", "/");
        assert_collision!("/a?<a>", "/a");
        assert_collision!("/a?<a>", "/a");
        assert_collision!("/a/b?<a>", "/a/b");
        assert_collision!("/a/b", "/a/b?<c>");

        assert_collision!("/a/hi/<a..>", "/a/hi/");
        assert_collision!("/hi/<a..>", "/hi/");
        assert_collision!("/<a..>", "/");
    }

    fn mt_mt_collide(mt1: &str, mt2: &str) -> bool {
        let mt_a = MediaType::from_str(mt1).expect(mt1);
        let mt_b = MediaType::from_str(mt2).expect(mt2);
        mt_a.collides_with(&mt_b)
    }

    #[test]
    fn test_content_type_collisions() {
        assert!(mt_mt_collide("application/json", "application/json"));
        assert!(mt_mt_collide("*/json", "application/json"));
        assert!(mt_mt_collide("*/*", "application/json"));
        assert!(mt_mt_collide("application/*", "application/json"));
        assert!(mt_mt_collide("application/*", "*/json"));
        assert!(mt_mt_collide("something/random", "something/random"));

        assert!(!mt_mt_collide("text/*", "application/*"));
        assert!(!mt_mt_collide("*/text", "*/json"));
        assert!(!mt_mt_collide("*/text", "application/test"));
        assert!(!mt_mt_collide("something/random", "something_else/random"));
        assert!(!mt_mt_collide("something/random", "*/else"));
        assert!(!mt_mt_collide("*/random", "*/else"));
        assert!(!mt_mt_collide("something/*", "random/else"));
    }

    fn r_mt_mt_collide<S1, S2>(m: Method, mt1: S1, mt2: S2) -> bool
        where S1: Into<Option<&'static str>>, S2: Into<Option<&'static str>>
    {
        let mut route_a = Route::new(m, "/", dummy_handler);
        if let Some(mt_str) = mt1.into() {
            route_a.format = Some(mt_str.parse::<MediaType>().unwrap());
        }

        let mut route_b = Route::new(m, "/", dummy_handler);
        if let Some(mt_str) = mt2.into() {
            route_b.format = Some(mt_str.parse::<MediaType>().unwrap());
        }

        route_a.collides_with(&route_b)
    }

    #[test]
    fn test_route_content_type_collisions() {
        // non-payload bearing routes always collide
        assert!(r_mt_mt_collide(Get, "application/json", "application/json"));
        assert!(r_mt_mt_collide(Get, "*/json", "application/json"));
        assert!(r_mt_mt_collide(Get, "*/json", "application/*"));
        assert!(r_mt_mt_collide(Get, "text/html", "text/*"));
        assert!(r_mt_mt_collide(Get, "any/thing", "*/*"));

        assert!(r_mt_mt_collide(Get, None, "text/*"));
        assert!(r_mt_mt_collide(Get, None, "text/html"));
        assert!(r_mt_mt_collide(Get, None, "*/*"));
        assert!(r_mt_mt_collide(Get, "text/html", None));
        assert!(r_mt_mt_collide(Get, "*/*", None));
        assert!(r_mt_mt_collide(Get, "application/json", None));

        assert!(r_mt_mt_collide(Get, "application/*", "text/*"));
        assert!(r_mt_mt_collide(Get, "application/json", "text/*"));
        assert!(r_mt_mt_collide(Get, "application/json", "text/html"));
        assert!(r_mt_mt_collide(Get, "text/html", "text/html"));

        // payload bearing routes collide if the media types collide
        assert!(r_mt_mt_collide(Post, "application/json", "application/json"));
        assert!(r_mt_mt_collide(Post, "*/json", "application/json"));
        assert!(r_mt_mt_collide(Post, "*/json", "application/*"));
        assert!(r_mt_mt_collide(Post, "text/html", "text/*"));
        assert!(r_mt_mt_collide(Post, "any/thing", "*/*"));

        assert!(r_mt_mt_collide(Post, None, "text/*"));
        assert!(r_mt_mt_collide(Post, None, "text/html"));
        assert!(r_mt_mt_collide(Post, None, "*/*"));
        assert!(r_mt_mt_collide(Post, "text/html", None));
        assert!(r_mt_mt_collide(Post, "*/*", None));
        assert!(r_mt_mt_collide(Post, "application/json", None));

        assert!(!r_mt_mt_collide(Post, "text/html", "application/*"));
        assert!(!r_mt_mt_collide(Post, "application/html", "text/*"));
        assert!(!r_mt_mt_collide(Post, "*/json", "text/html"));
        assert!(!r_mt_mt_collide(Post, "text/html", "text/css"));
        assert!(!r_mt_mt_collide(Post, "other/html", "text/html"));
    }

    fn catchers_collide<A, B>(a: A, ap: &str, b: B, bp: &str) -> bool
        where A: Into<Option<u16>>, B: Into<Option<u16>>
    {
        use crate::catcher::dummy_handler as handler;

        let a = Catcher::new(a, handler).map_base(|_| ap.into()).unwrap();
        let b = Catcher::new(b, handler).map_base(|_| bp.into()).unwrap();
        a.collides_with(&b)
    }

    #[test]
    fn catcher_collisions() {
        for path in &["/a", "/foo", "/a/b/c", "/a/b/c/d/e"] {
            assert!(catchers_collide(404, path, 404, path));
            assert!(catchers_collide(500, path, 500, path));
            assert!(catchers_collide(None, path, None, path));
        }
    }

    #[test]
    fn catcher_non_collisions() {
        assert!(!catchers_collide(404, "/foo", 405, "/foo"));
        assert!(!catchers_collide(404, "/", None, "/foo"));
        assert!(!catchers_collide(404, "/", None, "/"));
        assert!(!catchers_collide(404, "/a/b", None, "/a/b"));
        assert!(!catchers_collide(404, "/a/b", 404, "/a/b/c"));

        assert!(!catchers_collide(None, "/a/b", None, "/a/b/c"));
        assert!(!catchers_collide(None, "/b", None, "/a/b/c"));
        assert!(!catchers_collide(None, "/", None, "/a/b/c"));
    }
}
