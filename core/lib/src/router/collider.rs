use super::Route;

use http::uri::Origin;
use http::MediaType;
use request::Request;

impl Route {
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
    /// Query collisions work like this:
    ///
    ///   * If routes specify a query, they only gets request that have queries.
    ///   * If routes don't specify a query, requests with queries also match.
    ///
    /// As a result, as long as everything else collides, whether a route has a
    /// query or not is irrelevant: it will collide.
    pub fn collides_with(&self, other: &Route) -> bool {
        self.method == other.method
            && self.rank == other.rank
            && paths_collide(&self.uri, &other.uri)
            && match (self.format.as_ref(), other.format.as_ref()) {
                (Some(a), Some(b)) => media_types_collide(a, b),
                (Some(_), None) => true,
                (None, Some(_)) => true,
                (None, None) => true
            }
    }

    /// Determines if this route matches against the given request. This means
    /// that:
    ///
    ///   * The route's method matches that of the incoming request.
    ///   * The route's format (if any) matches that of the incoming request.
    ///     - If route specifies format, it only gets requests for that format.
    ///     - If route doesn't specify format, it gets requests for any format.
    ///   * All static components in the route's path match the corresponding
    ///     components in the same position in the incoming request.
    ///   * If the route specifies a query, the request must have a query as
    ///     well. If the route doesn't specify a query, requests with and
    ///     without queries match.
    ///
    /// In the future, query handling will work as follows:
    ///
    ///   * All static components in the route's query string are also in the
    ///     request query string, though in any position, and there exists a
    ///     query parameter named exactly like each non-multi dynamic component
    ///     in the route's query that wasn't matched against a static component.
    ///     - If no query in route, requests with/without queries match.
    pub fn matches(&self, req: &Request) -> bool {
        self.method == req.method()
            && paths_collide(&self.uri, req.uri())
            && queries_collide(self, req)
            && match self.format {
                Some(ref a) => match req.format() {
                    Some(ref b) => media_types_collide(a, b),
                    None => false
                },
                None => true
            }
    }
}

#[inline(always)]
fn iters_match_until<A, B>(break_c: u8, mut a: A, mut b: B) -> bool
    where A: Iterator<Item = u8>, B: Iterator<Item = u8>
{
    loop {
        match (a.next(), b.next()) {
            (None, Some(_)) => return false,
            (Some(_), None) => return false,
            (None, None) => return true,
            (Some(c1), Some(c2)) if c1 == break_c || c2 == break_c => return true,
            (Some(c1), Some(c2)) if c1 != c2 => return false,
            (Some(_), Some(_)) => continue
        }
    }
}

fn segments_collide(first: &str, other: &str) -> bool {
    let a_iter = first.as_bytes().iter().cloned();
    let b_iter = other.as_bytes().iter().cloned();
    iters_match_until(b'<', a_iter.clone(), b_iter.clone())
        && iters_match_until(b'>', a_iter.rev(), b_iter.rev())
}

fn paths_collide(first: &Origin, other: &Origin) -> bool {
    for (seg_a, seg_b) in first.segments().zip(other.segments()) {
        if seg_a.ends_with("..>") || seg_b.ends_with("..>") {
            return true;
        }

        if !segments_collide(seg_a, seg_b) {
            return false;
        }
    }

    if first.segment_count() != other.segment_count() {
        return false;
    }

    true
}

fn queries_collide(route: &Route, req: &Request) -> bool {
    route.uri.query().map_or(true, |_| req.uri().query().is_some())
}

fn media_types_collide(first: &MediaType, other: &MediaType) -> bool {
    let collide = |a, b| a == "*" || b == "*" || a == b;
    collide(first.top(), other.top()) && collide(first.sub(), other.sub())
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use rocket::Rocket;
    use config::Config;
    use request::Request;
    use router::{dummy_handler, route::Route};
    use http::{Method, MediaType, ContentType, Accept};
    use http::uri::Origin;
    use http::Method::*;

    type SimpleRoute = (Method, &'static str);

    fn m_collide(a: SimpleRoute, b: SimpleRoute) -> bool {
        let route_a = Route::new(a.0, a.1.to_string(), dummy_handler);
        route_a.collides_with(&Route::new(b.0, b.1.to_string(), dummy_handler))
    }

    fn unranked_collide(a: &'static str, b: &'static str) -> bool {
        let route_a = Route::ranked(0, Get, a.to_string(), dummy_handler);
        let route_b = Route::ranked(0, Get, b.to_string(), dummy_handler);
        eprintln!("Checking {} against {}.", route_a, route_b);
        route_a.collides_with(&route_b)
    }

    fn s_s_collide(a: &'static str, b: &'static str) -> bool {
        let a = Origin::parse_route(a).unwrap();
        let b = Origin::parse_route(b).unwrap();
        paths_collide(&a, &b)
    }

    #[test]
    fn simple_collisions() {
        assert!(unranked_collide("/a", "/a"));
        assert!(unranked_collide("/hello", "/hello"));
        assert!(unranked_collide("/hello", "/hello/"));
        assert!(unranked_collide("/hello/there/how/ar", "/hello/there/how/ar"));
        assert!(unranked_collide("/hello/there", "/hello/there/"));
    }

    #[test]
    fn simple_param_collisions() {
        assert!(unranked_collide("/hello/<name>", "/hello/<person>"));
        assert!(unranked_collide("/hello/<name>/hi", "/hello/<person>/hi"));
        assert!(unranked_collide("/hello/<name>/hi/there", "/hello/<person>/hi/there"));
        assert!(unranked_collide("/<name>/hi/there", "/<person>/hi/there"));
        assert!(unranked_collide("/<name>/hi/there", "/dude/<name>/there"));
        assert!(unranked_collide("/<name>/<a>/<b>", "/<a>/<b>/<c>"));
        assert!(unranked_collide("/<name>/<a>/<b>/", "/<a>/<b>/<c>/"));
        assert!(unranked_collide("/<a..>", "/hi"));
        assert!(unranked_collide("/<a..>", "/hi/hey"));
        assert!(unranked_collide("/<a..>", "/hi/hey/hayo"));
        assert!(unranked_collide("/a/<a..>", "/a/hi/hey/hayo"));
        assert!(unranked_collide("/a/<b>/<a..>", "/a/hi/hey/hayo"));
        assert!(unranked_collide("/a/<b>/<c>/<a..>", "/a/hi/hey/hayo"));
        assert!(unranked_collide("/<b>/<c>/<a..>", "/a/hi/hey/hayo"));
        assert!(unranked_collide("/<b>/<c>/hey/hayo", "/a/hi/hey/hayo"));
    }

    #[test]
    fn medium_param_collisions() {
        assert!(unranked_collide("/hello/<name>", "/hello/bob"));
        assert!(unranked_collide("/<name>", "//bob"));
    }

    #[test]
    fn hard_param_collisions() {
        assert!(unranked_collide("/<name>bob", "/<name>b"));
        assert!(unranked_collide("/a<b>c", "/abc"));
        assert!(unranked_collide("/a<b>c", "/azooc"));
        assert!(unranked_collide("/a<b>", "/ab"));
        assert!(unranked_collide("/<b>", "/a"));
        assert!(unranked_collide("/<a>/<b>", "/a/b<c>"));
        assert!(unranked_collide("/<a>/bc<b>", "/a/b<c>"));
        assert!(unranked_collide("/<a>/bc<b>d", "/a/b<c>"));
        assert!(unranked_collide("/<a..>", "///a///"));
    }

    #[test]
    fn query_collisions() {
        assert!(unranked_collide("/?<a>", "/?<a>"));
        assert!(unranked_collide("/a/?<a>", "/a/?<a>"));
        assert!(unranked_collide("/a?<a>", "/a?<a>"));
        assert!(unranked_collide("/<r>?<a>", "/<r>?<a>"));
        assert!(unranked_collide("/a/b/c?<a>", "/a/b/c?<a>"));
        assert!(unranked_collide("/<a>/b/c?<d>", "/a/b/<c>?<d>"));
        assert!(unranked_collide("/?<a>", "/"));
        assert!(unranked_collide("/a?<a>", "/a"));
        assert!(unranked_collide("/a?<a>", "/a"));
        assert!(unranked_collide("/a/b?<a>", "/a/b"));
        assert!(unranked_collide("/a/b", "/a/b?<c>"));
    }

    #[test]
    fn non_collisions() {
        assert!(!unranked_collide("/<a>", "/"));
        assert!(!unranked_collide("/a", "/b"));
        assert!(!unranked_collide("/a/b", "/a"));
        assert!(!unranked_collide("/a/b", "/a/c"));
        assert!(!unranked_collide("/a/hello", "/a/c"));
        assert!(!unranked_collide("/hello", "/a/c"));
        assert!(!unranked_collide("/hello/there", "/hello/there/guy"));
        assert!(!unranked_collide("/b<a>/there", "/hi/there"));
        assert!(!unranked_collide("/<a>/<b>c", "/hi/person"));
        assert!(!unranked_collide("/<a>/<b>cd", "/hi/<a>e"));
        assert!(!unranked_collide("/a<a>/<b>", "/b<b>/<a>"));
        assert!(!unranked_collide("/a/<b>", "/b/<b>"));
        assert!(!unranked_collide("/a<a>/<b>", "/b/<b>"));
        assert!(!unranked_collide("/<a..>", "/"));
        assert!(!unranked_collide("/hi/<a..>", "/hi"));
        assert!(!unranked_collide("/hi/<a..>", "/hi/"));
        assert!(!unranked_collide("/<a..>", "//////"));
        assert!(!unranked_collide("/t", "/test"));
        assert!(!unranked_collide("/a", "/aa"));
        assert!(!unranked_collide("/a", "/aaa"));
        assert!(!unranked_collide("/", "/a"));
    }

    #[test]
    fn query_non_collisions() {
        assert!(!unranked_collide("/a?<b>", "/b"));
        assert!(!unranked_collide("/a/b", "/a?<b>"));
        assert!(!unranked_collide("/a/b/c?<d>", "/a/b/c/d"));
        assert!(!unranked_collide("/a/hello", "/a/?<hello>"));
        assert!(!unranked_collide("/?<a>", "/hi"));
    }

    #[test]
    fn method_dependent_non_collisions() {
        assert!(!m_collide((Get, "/"), (Post, "/")));
        assert!(!m_collide((Post, "/"), (Put, "/")));
        assert!(!m_collide((Put, "/a"), (Put, "/")));
        assert!(!m_collide((Post, "/a"), (Put, "/")));
        assert!(!m_collide((Get, "/a"), (Put, "/")));
        assert!(!m_collide((Get, "/hello"), (Put, "/hello")));
    }

    #[test]
    fn query_dependent_non_collisions() {
        assert!(!m_collide((Get, "/"), (Get, "/?a")));
        assert!(!m_collide((Get, "/"), (Get, "/?<a>")));
        assert!(!m_collide((Get, "/a/<b>"), (Get, "/a/<b>?d")));
    }

    #[test]
    fn test_str_non_collisions() {
        assert!(!s_s_collide("/a", "/b"));
        assert!(!s_s_collide("/a/b", "/a"));
        assert!(!s_s_collide("/a/b", "/a/c"));
        assert!(!s_s_collide("/a/hello", "/a/c"));
        assert!(!s_s_collide("/hello", "/a/c"));
        assert!(!s_s_collide("/hello/there", "/hello/there/guy"));
        assert!(!s_s_collide("/b<a>/there", "/hi/there"));
        assert!(!s_s_collide("/<a>/<b>c", "/hi/person"));
        assert!(!s_s_collide("/<a>/<b>cd", "/hi/<a>e"));
        assert!(!s_s_collide("/a<a>/<b>", "/b<b>/<a>"));
        assert!(!s_s_collide("/a/<b>", "/b/<b>"));
        assert!(!s_s_collide("/a<a>/<b>", "/b/<b>"));
        assert!(!s_s_collide("/a", "/b"));
        assert!(!s_s_collide("/a/b", "/a"));
        assert!(!s_s_collide("/a/b", "/a/c"));
        assert!(!s_s_collide("/a/hello", "/a/c"));
        assert!(!s_s_collide("/hello", "/a/c"));
        assert!(!s_s_collide("/hello/there", "/hello/there/guy"));
        assert!(!s_s_collide("/b<a>/there", "/hi/there"));
        assert!(!s_s_collide("/<a>/<b>c", "/hi/person"));
        assert!(!s_s_collide("/<a>/<b>cd", "/hi/<a>e"));
        assert!(!s_s_collide("/a<a>/<b>", "/b<b>/<a>"));
        assert!(!s_s_collide("/a/<b>", "/b/<b>"));
        assert!(!s_s_collide("/a<a>/<b>", "/b/<b>"));
        assert!(!s_s_collide("/a", "/b"));
        assert!(!s_s_collide("/a/b", "/a"));
        assert!(!s_s_collide("/a/b", "/a/c"));
        assert!(!s_s_collide("/a/hello", "/a/c"));
        assert!(!s_s_collide("/hello", "/a/c"));
        assert!(!s_s_collide("/hello/there", "/hello/there/guy"));
        assert!(!s_s_collide("/b<a>/there", "/hi/there"));
        assert!(!s_s_collide("/<a>/<b>c", "/hi/person"));
        assert!(!s_s_collide("/<a>/<b>cd", "/hi/<a>e"));
        assert!(!s_s_collide("/a<a>/<b>", "/b<b>/<a>"));
        assert!(!s_s_collide("/a/<b>", "/b/<b>"));
        assert!(!s_s_collide("/a<a>/<b>", "/b/<b>"));
        assert!(!s_s_collide("/<a..>", "/"));
        assert!(!s_s_collide("/hi/<a..>", "/hi/"));
        assert!(!s_s_collide("/a/hi/<a..>", "/a/hi/"));
        assert!(!s_s_collide("/t", "/test"));
        assert!(!s_s_collide("/a", "/aa"));
        assert!(!s_s_collide("/a", "/aaa"));
        assert!(!s_s_collide("/", "/a"));
    }

    fn mt_mt_collide(mt1: &str, mt2: &str) -> bool {
        let mt_a = MediaType::from_str(mt1).expect(mt1);
        let mt_b = MediaType::from_str(mt2).expect(mt2);
        media_types_collide(&mt_a, &mt_b)
    }

    #[test]
    fn test_content_type_colliions() {
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

    fn r_mt_mt_collide<S1, S2>(m1: Method, mt1: S1, m2: Method, mt2: S2) -> bool
        where S1: Into<Option<&'static str>>, S2: Into<Option<&'static str>>
    {
        let mut route_a = Route::new(m1, "/", dummy_handler);
        if let Some(mt_str) = mt1.into() {
            route_a.format = Some(mt_str.parse::<MediaType>().unwrap());
        }

        let mut route_b = Route::new(m2, "/", dummy_handler);
        if let Some(mt_str) = mt2.into() {
            route_b.format = Some(mt_str.parse::<MediaType>().unwrap());
        }

        route_a.collides_with(&route_b)
    }

    #[test]
    fn test_route_content_type_colliions() {
        assert!(r_mt_mt_collide(Get, "application/json", Get, "application/json"));
        assert!(r_mt_mt_collide(Get, "*/json", Get, "application/json"));
        assert!(r_mt_mt_collide(Get, "*/json", Get, "application/*"));
        assert!(r_mt_mt_collide(Get, "text/html", Get, "text/*"));
        assert!(r_mt_mt_collide(Get, "any/thing", Get, "*/*"));

        assert!(r_mt_mt_collide(Get, None, Get, "text/*"));
        assert!(r_mt_mt_collide(Get, None, Get, "text/html"));
        assert!(r_mt_mt_collide(Get, None, Get, "*/*"));
        assert!(r_mt_mt_collide(Get, "text/html", Get, None));
        assert!(r_mt_mt_collide(Get, "*/*", Get, None));
        assert!(r_mt_mt_collide(Get, "application/json", Get, None));

        assert!(!r_mt_mt_collide(Get, "text/html", Get, "application/*"));
        assert!(!r_mt_mt_collide(Get, "application/html", Get, "text/*"));
        assert!(!r_mt_mt_collide(Get, "*/json", Get, "text/html"));
        assert!(!r_mt_mt_collide(Get, "text/html", Get, "text/css"));
    }

    fn req_route_mt_collide<S1, S2>(m: Method, mt1: S1, mt2: S2) -> bool
        where S1: Into<Option<&'static str>>, S2: Into<Option<&'static str>>
    {
        let rocket = Rocket::custom(Config::development().unwrap());
        let mut req = Request::new(&rocket, m, Origin::dummy());
        if let Some(mt_str) = mt1.into() {
            if m.supports_payload() {
                req.replace_header(mt_str.parse::<ContentType>().unwrap());
            } else {
                req.replace_header(mt_str.parse::<Accept>().unwrap());
            }
        }

        let mut route = Route::new(m, "/", dummy_handler);
        if let Some(mt_str) = mt2.into() {
            route.format = Some(mt_str.parse::<MediaType>().unwrap());
        }

        route.matches(&req)
    }

    #[test]
    fn test_req_route_mt_collisions() {
        assert!(req_route_mt_collide(Post, "application/json", "application/json"));
        assert!(req_route_mt_collide(Post, "application/json", "application/*"));
        assert!(req_route_mt_collide(Post, "application/json", "*/json"));
        assert!(req_route_mt_collide(Post, "text/html", "*/*"));

        assert!(req_route_mt_collide(Get, "application/json", "application/json"));
        assert!(req_route_mt_collide(Get, "text/html", "text/html"));
        assert!(req_route_mt_collide(Get, "text/html", "*/*"));
        assert!(req_route_mt_collide(Get, None, "text/html"));
        assert!(req_route_mt_collide(Get, None, "*/*"));
        assert!(req_route_mt_collide(Get, None, "application/json"));

        assert!(req_route_mt_collide(Post, "text/html", None));
        assert!(req_route_mt_collide(Post, "application/json", None));
        assert!(req_route_mt_collide(Post, "x-custom/anything", None));
        assert!(req_route_mt_collide(Post, None, None));

        assert!(req_route_mt_collide(Get, "text/html", None));
        assert!(req_route_mt_collide(Get, "application/json", None));
        assert!(req_route_mt_collide(Get, "x-custom/anything", None));
        assert!(req_route_mt_collide(Get, None, None));

        assert!(req_route_mt_collide(Get, "text/html, text/plain", "text/html"));
        assert!(req_route_mt_collide(Get, "text/html; q=0.5, text/xml", "text/xml"));

        assert!(!req_route_mt_collide(Post, "application/json", "text/html"));
        assert!(!req_route_mt_collide(Post, "application/json", "text/*"));
        assert!(!req_route_mt_collide(Post, "application/json", "*/xml"));
        assert!(!req_route_mt_collide(Get, "application/json", "text/html"));
        assert!(!req_route_mt_collide(Get, "application/json", "text/*"));
        assert!(!req_route_mt_collide(Get, "application/json", "*/xml"));

        assert!(!req_route_mt_collide(Post, None, "text/html"));
        assert!(!req_route_mt_collide(Post, None, "*/*"));
        assert!(!req_route_mt_collide(Post, None, "application/json"));
    }

    fn req_route_path_collide(a: &'static str, b: &'static str) -> bool {
        let rocket = Rocket::custom(Config::development().unwrap());
        let req = Request::new(&rocket, Get, Origin::parse(a).expect("valid URI"));
        let route = Route::ranked(0, Get, b.to_string(), dummy_handler);
        route.matches(&req)
    }

    #[test]
    fn test_req_route_query_collisions() {
        assert!(req_route_path_collide("/a/b?a=b", "/a/b?<c>"));
        assert!(req_route_path_collide("/a/b?a=b", "/<a>/b?<c>"));
        assert!(req_route_path_collide("/a/b?a=b", "/<a>/<b>?<c>"));
        assert!(req_route_path_collide("/a/b?a=b", "/a/<b>?<c>"));
        assert!(req_route_path_collide("/?b=c", "/?<b>"));

        assert!(req_route_path_collide("/a/b?a=b", "/a/b"));
        assert!(req_route_path_collide("/a/b", "/a/b"));
        assert!(req_route_path_collide("/a/b/c/d?", "/a/b/c/d"));
        assert!(req_route_path_collide("/a/b/c/d?v=1&v=2", "/a/b/c/d"));

        assert!(!req_route_path_collide("/a/b", "/a/b?<c>"));
        assert!(!req_route_path_collide("/a/b/c", "/a/b?<c>"));
        assert!(!req_route_path_collide("/a?b=c", "/a/b?<c>"));
        assert!(!req_route_path_collide("/?b=c", "/a/b?<c>"));
        assert!(!req_route_path_collide("/?b=c", "/a?<c>"));
    }
}
