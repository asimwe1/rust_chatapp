use crate::{Route, Request, Catcher};
use crate::router::Collide;
use crate::http::Status;
use crate::route::Color;

impl Route {
    /// Determines if this route matches against the given request.
    ///
    /// This means that:
    ///
    ///   * The route's method matches that of the incoming request.
    ///   * The route's format (if any) matches that of the incoming request.
    ///     - If route specifies format, it only gets requests for that format.
    ///     - If route doesn't specify format, it gets requests for any format.
    ///   * All static components in the route's path match the corresponding
    ///     components in the same position in the incoming request.
    ///   * All static components in the route's query string are also in the
    ///     request query string, though in any position. If there is no query
    ///     in the route, requests with/without queries match.
    #[doc(hidden)]
    pub fn matches(&self, req: &Request<'_>) -> bool {
        self.method == req.method()
            && paths_match(self, req)
            && queries_match(self, req)
            && formats_match(self, req)
    }
}

impl Catcher {
    /// Determines if this catcher is responsible for handling the error with
    /// `status` that occurred during request `req`. A catcher matches if:
    ///
    ///  * It is a default catcher _or_ has a code of `status`.
    ///  * Its base is a prefix of the normalized/decoded `req.path()`.
    pub(crate) fn matches(&self, status: Status, req: &Request<'_>) -> bool {
        self.code.map_or(true, |code| code == status.code)
            && self.base.path().segments().prefix_of(req.uri().path().segments())
    }
}

fn paths_match(route: &Route, req: &Request<'_>) -> bool {
    trace!("checking path match: route {} vs. request {}", route, req);
    let route_segments = &route.uri.metadata.uri_segments;
    let req_segments = req.uri().path().segments();

    // A route can never have more segments than a request. Recall that a
    // trailing slash is considering a segment, albeit empty.
    if route_segments.len() > req_segments.num() {
        return false;
    }

    // requests with longer paths only match if we have dynamic trail (<a..>).
    if req_segments.num() > route_segments.len() {
        if !route.uri.metadata.dynamic_trail {
            return false;
        }
    }

    // We've checked everything beyond the zip of their lengths already.
    for (route_seg, req_seg) in route_segments.iter().zip(req_segments.clone()) {
        if route_seg.dynamic_trail {
            return true;
        }

        if !route_seg.dynamic && route_seg.value != req_seg {
            return false;
        }
    }

    true
}

fn queries_match(route: &Route, req: &Request<'_>) -> bool {
    trace!("checking query match: route {} vs. request {}", route, req);
    if matches!(route.uri.metadata.query_color, None | Some(Color::Wild)) {
        return true;
    }

    let route_query_fields = route.uri.metadata.static_query_fields.iter()
        .map(|(k, v)| (k.as_str(), v.as_str()));

    for route_seg in route_query_fields {
        if let Some(query) = req.uri().query() {
            if !query.segments().any(|req_seg| req_seg == route_seg) {
                trace_!("request {} missing static query {:?}", req, route_seg);
                return false;
            }
        } else {
            trace_!("query-less request {} missing static query {:?}", req, route_seg);
            return false;
        }
    }

    true
}

fn formats_match(route: &Route, req: &Request<'_>) -> bool {
    trace!("checking format match: route {} vs. request {}", route, req);
    let route_format = match route.format {
        Some(ref format) => format,
        None => return true,
    };

    if route.method.supports_payload() {
        match req.format() {
            Some(f) if f.specificity() == 2 => route_format.collides_with(f),
            _ => false
        }
    } else {
        match req.format() {
            Some(f) => route_format.collides_with(f),
            None => true
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::local::blocking::Client;
    use crate::route::{Route, dummy_handler};
    use crate::http::{Method, Method::*, MediaType, ContentType, Accept};

    fn req_matches_route(a: &'static str, b: &'static str) -> bool {
        let client = Client::debug_with(vec![]).expect("client");
        let route = Route::ranked(0, Get, b, dummy_handler);
        route.matches(&client.get(a))
    }

    #[test]
    fn request_route_matching() {
        assert!(req_matches_route("/a/b?a=b", "/a/b?<c>"));
        assert!(req_matches_route("/a/b?a=b", "/<a>/b?<c>"));
        assert!(req_matches_route("/a/b?a=b", "/<a>/<b>?<c>"));
        assert!(req_matches_route("/a/b?a=b", "/a/<b>?<c>"));
        assert!(req_matches_route("/?b=c", "/?<b>"));

        assert!(req_matches_route("/a/b?a=b", "/a/b"));
        assert!(req_matches_route("/a/b", "/a/b"));
        assert!(req_matches_route("/a/b/c/d?", "/a/b/c/d"));
        assert!(req_matches_route("/a/b/c/d?v=1&v=2", "/a/b/c/d"));

        assert!(req_matches_route("/a/b", "/a/b?<c>"));
        assert!(req_matches_route("/a/b", "/a/b?<c..>"));
        assert!(req_matches_route("/a/b?c", "/a/b?c"));
        assert!(req_matches_route("/a/b?c", "/a/b?<c>"));
        assert!(req_matches_route("/a/b?c=foo&d=z", "/a/b?<c>"));
        assert!(req_matches_route("/a/b?c=foo&d=z", "/a/b?<c..>"));
        assert!(req_matches_route("/a/b?c=foo&d=z", "/a/b?c=foo&<c..>"));
        assert!(req_matches_route("/a/b?c=foo&d=z", "/a/b?d=z&<c..>"));

        assert!(req_matches_route("/", "/<foo>"));
        assert!(req_matches_route("/a", "/<foo>"));
        assert!(req_matches_route("/a", "/a"));
        assert!(req_matches_route("/a/", "/a/"));

        assert!(req_matches_route("//", "/"));
        assert!(req_matches_route("/a///", "/a/"));
        assert!(req_matches_route("/a/b", "/a/b"));

        assert!(!req_matches_route("/a///", "/a"));
        assert!(!req_matches_route("/a", "/a/"));
        assert!(!req_matches_route("/a/", "/a"));
        assert!(!req_matches_route("/a/b", "/a/b/"));

        assert!(!req_matches_route("/a", "/<a>/"));
        assert!(!req_matches_route("/a/", "/<a>"));
        assert!(!req_matches_route("/a/b", "/<a>/b/"));
        assert!(!req_matches_route("/a/b", "/<a>/<b>/"));

        assert!(!req_matches_route("/a/b/c", "/a/b?<c>"));
        assert!(!req_matches_route("/a?b=c", "/a/b?<c>"));
        assert!(!req_matches_route("/?b=c", "/a/b?<c>"));
        assert!(!req_matches_route("/?b=c", "/a?<c>"));

        assert!(!req_matches_route("/a/", "/<a>/<b>/<c..>"));
        assert!(!req_matches_route("/a/b", "/<a>/<b>/<c..>"));

        assert!(!req_matches_route("/a/b?c=foo&d=z", "/a/b?a=b&<c..>"));
        assert!(!req_matches_route("/a/b?c=foo&d=z", "/a/b?d=b&<c..>"));
        assert!(!req_matches_route("/a/b", "/a/b?c"));
        assert!(!req_matches_route("/a/b", "/a/b?foo"));
        assert!(!req_matches_route("/a/b", "/a/b?foo&<rest..>"));
        assert!(!req_matches_route("/a/b", "/a/b?<a>&b&<rest..>"));
    }

    fn req_matches_format<S1, S2>(m: Method, mt1: S1, mt2: S2) -> bool
        where S1: Into<Option<&'static str>>, S2: Into<Option<&'static str>>
    {
        let client = Client::debug_with(vec![]).expect("client");
        let mut req = client.req(m, "/");
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
        assert!(req_matches_format(Post, "application/json", "application/json"));
        assert!(req_matches_format(Post, "application/json", "application/*"));
        assert!(req_matches_format(Post, "application/json", "*/json"));
        assert!(req_matches_format(Post, "text/html", "*/*"));

        assert!(req_matches_format(Get, "application/json", "application/json"));
        assert!(req_matches_format(Get, "text/html", "text/html"));
        assert!(req_matches_format(Get, "text/html", "*/*"));
        assert!(req_matches_format(Get, None, "*/*"));
        assert!(req_matches_format(Get, None, "text/*"));
        assert!(req_matches_format(Get, None, "text/html"));
        assert!(req_matches_format(Get, None, "application/json"));

        assert!(req_matches_format(Post, "text/html", None));
        assert!(req_matches_format(Post, "application/json", None));
        assert!(req_matches_format(Post, "x-custom/anything", None));
        assert!(req_matches_format(Post, None, None));

        assert!(req_matches_format(Get, "text/html", None));
        assert!(req_matches_format(Get, "application/json", None));
        assert!(req_matches_format(Get, "x-custom/anything", None));
        assert!(req_matches_format(Get, None, None));
        assert!(req_matches_format(Get, None, "text/html"));
        assert!(req_matches_format(Get, None, "application/json"));

        assert!(req_matches_format(Get, "text/html, text/plain", "text/html"));
        assert!(req_matches_format(Get, "text/html; q=0.5, text/xml", "text/xml"));

        assert!(!req_matches_format(Post, None, "text/html"));
        assert!(!req_matches_format(Post, None, "text/*"));
        assert!(!req_matches_format(Post, None, "*/text"));
        assert!(!req_matches_format(Post, None, "*/*"));
        assert!(!req_matches_format(Post, None, "text/html"));
        assert!(!req_matches_format(Post, None, "application/json"));

        assert!(!req_matches_format(Post, "application/json", "text/html"));
        assert!(!req_matches_format(Post, "application/json", "text/*"));
        assert!(!req_matches_format(Post, "application/json", "*/xml"));
        assert!(!req_matches_format(Get, "application/json", "text/html"));
        assert!(!req_matches_format(Get, "application/json", "text/*"));
        assert!(!req_matches_format(Get, "application/json", "*/xml"));

        assert!(!req_matches_format(Post, None, "text/html"));
        assert!(!req_matches_format(Post, None, "application/json"));
    }
}
