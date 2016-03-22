mod collider;
mod route;

pub use self::collider::Collider;

use term_painter::ToStyle;
use term_painter::Color::*;
use self::route::Route;
use std::collections::hash_map::HashMap;
use std::path::Path;
use method::Method;
use Handler;

type Selector = (Method, usize);

pub struct Router {
    routes: HashMap<Selector, Vec<Route>> // for now
}

impl Router {
    pub fn new() -> Router {
        Router {
            routes: HashMap::new()
        }
    }

    // FIXME: Take in Handler.
    pub fn add_route(&mut self, method: Method, base: &'static str,
                     route: &'static str, handler: Handler<'static>) {
        let route = Route::new(method, base, route, handler);
        let selector = (method, route.component_count());
        self.routes.entry(selector).or_insert(vec![]).push(route);
    }

    // TODO: Make a `Router` trait with this function. Rename this `Router`
    // struct to something like `RocketRouter`.
    pub fn route<'b>(&'b self, method: Method, uri: &str) -> Option<&'b Route> {
        let mut matched_route = None;
        let path = Path::new(uri);
        let num_components = path.components().count();
        if let Some(routes) = self.routes.get(&(method, num_components)) {
            for route in routes.iter().filter(|r| r.collides_with(uri)) {
                println!("Matched {} to: {}", uri, route);
                if let None = matched_route {
                    matched_route = Some(route);
                }
            }
        }

        matched_route
    }

    pub fn has_collisions(&self) -> bool {
        let mut result = false;
        for (_, routes) in &self.routes {
            for i in 0..routes.len() {
                for j in (i + 1)..routes.len() {
                    let (a_route, b_route) = (&routes[i], &routes[j]);
                    if a_route.collides_with(b_route) {
                        result = true;
                        println!("{} {} and {} collide!",
                            Yellow.bold().paint("Warning:"), a_route, b_route);
                    }
                }
            }
        }

        result
    }
}

#[cfg(test)]
mod test {
    use super::Router;
    use Method::*;
    use {Response, Request};

    fn dummy_handler(_req: Request) -> Response<'static> {
        Response::empty()
    }

    fn router_with_routes(routes: &[&'static str]) -> Router {
        let mut router = Router::new();
        for route in routes {
            router.add_route(Get, "/", route, dummy_handler);
        }

        router
    }

    #[test]
    fn test_collisions() {
        let router = router_with_routes(&["/hello", "/hello"]);
        assert!(router.has_collisions());

        let router = router_with_routes(&["/<a>", "/hello"]);
        assert!(router.has_collisions());

        let router = router_with_routes(&["/<a>", "/<b>"]);
        assert!(router.has_collisions());

        let router = router_with_routes(&["/hello/bob", "/hello/<b>"]);
        assert!(router.has_collisions());
    }

    #[test]
    fn test_ok_routing() {
        let router = router_with_routes(&["/hello"]);
        assert!(router.route(Get, "/hello").is_some());

        let router = router_with_routes(&["/<a>"]);
        assert!(router.route(Get, "/hello").is_some());
        assert!(router.route(Get, "/hi").is_some());
        assert!(router.route(Get, "/bobbbbbbbbbby").is_some());
        assert!(router.route(Get, "/dsfhjasdf").is_some());

        let router = router_with_routes(&["/<a>/<b>"]);
        assert!(router.route(Get, "/hello/hi").is_some());
        assert!(router.route(Get, "/a/b/").is_some());
        assert!(router.route(Get, "/i/a").is_some());
        assert!(router.route(Get, "/jdlk/asdij").is_some());

        let mut router = Router::new();
        router.add_route(Put, "/", "/hello", dummy_handler);
        router.add_route(Post, "/", "/hello", dummy_handler);
        router.add_route(Delete, "/", "/hello", dummy_handler);
        assert!(router.route(Put, "/hello").is_some());
        assert!(router.route(Post, "/hello").is_some());
        assert!(router.route(Delete, "/hello").is_some());
    }

    #[test]
    fn test_err_routing() {
        let router = router_with_routes(&["/hello"]);
        assert!(router.route(Put, "/hello").is_none());
        assert!(router.route(Post, "/hello").is_none());
        assert!(router.route(Options, "/hello").is_none());
        assert!(router.route(Get, "/hell").is_none());
        assert!(router.route(Get, "/hi").is_none());
        assert!(router.route(Get, "/hello/there").is_none());
        assert!(router.route(Get, "/hello/i").is_none());
        assert!(router.route(Get, "/hillo").is_none());

        let router = router_with_routes(&["/<a>"]);
        assert!(router.route(Put, "/hello").is_none());
        assert!(router.route(Post, "/hello").is_none());
        assert!(router.route(Options, "/hello").is_none());
        assert!(router.route(Get, "/hello/there").is_none());
        assert!(router.route(Get, "/hello/i").is_none());

        let router = router_with_routes(&["/<a>/<b>"]);
        assert!(router.route(Get, "/a/b/c").is_none());
        assert!(router.route(Get, "/a").is_none());
        assert!(router.route(Get, "/a/").is_none());
        assert!(router.route(Get, "/a/b/c/d").is_none());
        assert!(router.route(Put, "/hello/hi").is_none());
        assert!(router.route(Put, "/a/b").is_none());
        assert!(router.route(Put, "/a/b").is_none());
    }

    fn match_params(router: &Router, path: &str, expected: &[&str]) -> bool {
        router.route(Get, path).map_or(false, |route| {
            let params = route.get_params(path);
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
