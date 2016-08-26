mod collider;
mod route;
mod uri;

pub use self::collider::Collider;
pub use self::uri::{URI, URIBuf};
pub use self::route::Route;

use std::collections::hash_map::HashMap;
use method::Method;
use request::Request;

type Selector = (Method, usize);

#[derive(Default)]
pub struct Router {
    routes: HashMap<Selector, Vec<Route>> // using 'selector' for now
}

impl Router {
    pub fn new() -> Router {
        Router { routes: HashMap::new() }
    }

    pub fn add(&mut self, route: Route) {
        let selector = (route.method, route.path.segment_count());
        self.routes.entry(selector).or_insert_with(|| vec![]).push(route);
    }

    // TODO: Make a `Router` trait with this function. Rename this `Router`
    // struct to something like `RocketRouter`. If that happens, returning a
    // `Route` structure is inflexible. Have it be an associated type.
    // FIXME: Figure out a way to get more than one route, i.e., to correctly
    // handle ranking.
    // TODO: Should the Selector include the content-type? If it does, can't
    // warn the user that a match was found for the wrong content-type. It
    // doesn't, can, but this method is slower.
    pub fn route<'b>(&'b self, req: &Request) -> Option<&'b Route> {
        let num_segments = req.uri.segment_count();

        let mut matched_route: Option<&Route> = None;
        if let Some(routes) = self.routes.get(&(req.method, num_segments)) {
            for route in routes.iter().filter(|r| r.collides_with(req)) {
                info_!("Matched: {}", route);
                if let Some(existing_route) = matched_route {
                    if route.rank > existing_route.rank {
                        matched_route = Some(route);
                    }
                } else {
                    matched_route = Some(route);
                }
            }
        }

        matched_route
    }

    pub fn has_collisions(&self) -> bool {
        let mut result = false;
        for routes in self.routes.values() {
            for (i, a_route) in routes.iter().enumerate() {
                for b_route in routes.iter().skip(i + 1) {
                    if a_route.collides_with(b_route) {
                        result = true;
                        warn!("{} and {} collide!", a_route, b_route);
                    }
                }
            }
        }

        result
    }
}

#[cfg(test)]
mod test {
    use method::Method;
    use method::Method::*;
    use super::{Router, Route};
    use {Response, Request};
    use super::URI;

    fn dummy_handler(_req: &Request) -> Response<'static> {
        Response::empty()
    }

    fn router_with_routes(routes: &[&'static str]) -> Router {
        let mut router = Router::new();
        for route in routes {
            let route = Route::new(Get, route.to_string(), dummy_handler);
            router.add(route);
        }

        router
    }

    fn router_with_unranked_routes(routes: &[&'static str]) -> Router {
        let mut router = Router::new();
        for route in routes {
            let route = Route::ranked(0, Get, route.to_string(), dummy_handler);
            router.add(route);
        }

        router
    }

    #[test]
    fn test_collisions() {
        let router = router_with_unranked_routes(&["/hello", "/hello"]);
        assert!(router.has_collisions());

        let router = router_with_unranked_routes(&["/<a>", "/hello"]);
        assert!(router.has_collisions());

        let router = router_with_unranked_routes(&["/<a>", "/<b>"]);
        assert!(router.has_collisions());

        let router = router_with_unranked_routes(&["/hello/bob", "/hello/<b>"]);
        assert!(router.has_collisions());

        let router = router_with_routes(&["/a/b/<c>/d", "/<a>/<b>/c/d"]);
        assert!(router.has_collisions());
    }

    #[test]
    fn test_none_collisions_when_ranked() {
        let router = router_with_routes(&["/<a>", "/hello"]);
        assert!(!router.has_collisions());

        let router = router_with_routes(&["/hello/bob", "/hello/<b>"]);
        assert!(!router.has_collisions());

        let router = router_with_routes(&["/a/b/c/d", "/<a>/<b>/c/d"]);
        assert!(!router.has_collisions());

        let router = router_with_routes(&["/hi", "/<hi>"]);
        assert!(!router.has_collisions());
    }

    fn route<'a>(router: &'a Router, method: Method, uri: &str) -> Option<&'a Route> {
        let request = Request::mock(method, uri);
        router.route(&request)
    }

    #[test]
    fn test_ok_routing() {
        let router = router_with_routes(&["/hello"]);
        assert!(route(&router, Get, "/hello").is_some());

        let router = router_with_routes(&["/<a>"]);
        assert!(route(&router, Get, "/hello").is_some());
        assert!(route(&router, Get, "/hi").is_some());
        assert!(route(&router, Get, "/bobbbbbbbbbby").is_some());
        assert!(route(&router, Get, "/dsfhjasdf").is_some());

        let router = router_with_routes(&["/<a>/<b>"]);
        assert!(route(&router, Get, "/hello/hi").is_some());
        assert!(route(&router, Get, "/a/b/").is_some());
        assert!(route(&router, Get, "/i/a").is_some());
        assert!(route(&router, Get, "/jdlk/asdij").is_some());

        let mut router = Router::new();
        router.add(Route::new(Put, "/hello".to_string(), dummy_handler));
        router.add(Route::new(Post, "/hello".to_string(), dummy_handler));
        router.add(Route::new(Delete, "/hello".to_string(), dummy_handler));
        assert!(route(&router, Put, "/hello").is_some());
        assert!(route(&router, Post, "/hello").is_some());
        assert!(route(&router, Delete, "/hello").is_some());
    }

    #[test]
    fn test_err_routing() {
        let router = router_with_routes(&["/hello"]);
        assert!(route(&router, Put, "/hello").is_none());
        assert!(route(&router, Post, "/hello").is_none());
        assert!(route(&router, Options, "/hello").is_none());
        assert!(route(&router, Get, "/hell").is_none());
        assert!(route(&router, Get, "/hi").is_none());
        assert!(route(&router, Get, "/hello/there").is_none());
        assert!(route(&router, Get, "/hello/i").is_none());
        assert!(route(&router, Get, "/hillo").is_none());

        let router = router_with_routes(&["/<a>"]);
        assert!(route(&router, Put, "/hello").is_none());
        assert!(route(&router, Post, "/hello").is_none());
        assert!(route(&router, Options, "/hello").is_none());
        assert!(route(&router, Get, "/hello/there").is_none());
        assert!(route(&router, Get, "/hello/i").is_none());

        let router = router_with_routes(&["/<a>/<b>"]);
        assert!(route(&router, Get, "/a/b/c").is_none());
        assert!(route(&router, Get, "/a").is_none());
        assert!(route(&router, Get, "/a/").is_none());
        assert!(route(&router, Get, "/a/b/c/d").is_none());
        assert!(route(&router, Put, "/hello/hi").is_none());
        assert!(route(&router, Put, "/a/b").is_none());
        assert!(route(&router, Put, "/a/b").is_none());
    }

    macro_rules! assert_ranked_routes {
        ($routes:expr, $to:expr, $want:expr) => ({
            let router = router_with_routes($routes);
            let route_path = route(&router, Get, $to).unwrap().path.as_str();
            assert_eq!(route_path as &str, $want as &str);
        })
    }

    #[test]
    fn test_default_ranking() {
        assert_ranked_routes!(&["/hello", "/<name>"], "/hello", "/hello");
        assert_ranked_routes!(&["/<name>", "/hello"], "/hello", "/hello");
        assert_ranked_routes!(&["/<a>", "/hi", "/<b>"], "/hi", "/hi");
        assert_ranked_routes!(&["/<a>/b", "/hi/c"], "/hi/c", "/hi/c");
        assert_ranked_routes!(&["/<a>/<b>", "/hi/a"], "/hi/c", "/<a>/<b>");
        assert_ranked_routes!(&["/hi/a", "/hi/<c>"], "/hi/c", "/hi/<c>");
    }

    #[test]
    fn test_ranking() {
        // FIXME: Add tests for non-default ranks.
    }

    fn match_params(router: &Router, path: &str, expected: &[&str]) -> bool {
        route(router, Get, path).map_or(false, |route| {
            let params = route.get_params(URI::new(path));
            if params.len() != expected.len() {
                return false;
            }

            for i in 0..params.len() {
                if params[i] != expected[i] {
                    return false;
                }
            }

            true
        })
    }

    #[test]
    fn test_params() {
        let router = router_with_routes(&["/<a>"]);
        assert!(match_params(&router, "/hello", &["hello"]));
        assert!(match_params(&router, "/hi", &["hi"]));
        assert!(match_params(&router, "/bob", &["bob"]));
        assert!(match_params(&router, "/i", &["i"]));

        let router = router_with_routes(&["/hello"]);
        assert!(match_params(&router, "/hello", &[]));

        let router = router_with_routes(&["/<a>/<b>"]);
        assert!(match_params(&router, "/a/b", &["a", "b"]));
        assert!(match_params(&router, "/912/sas", &["912", "sas"]));

        let router = router_with_routes(&["/hello/<b>"]);
        assert!(match_params(&router, "/hello/b", &["b"]));
        assert!(match_params(&router, "/hello/sergio", &["sergio"]));

        let router = router_with_routes(&["/hello/<b>/age"]);
        assert!(match_params(&router, "/hello/sergio/age", &["sergio"]));
        assert!(match_params(&router, "/hello/you/age", &["you"]));
    }
}
