use method::Method;
use std::path::{PathBuf, Component};
use std::collections::hash_map::HashMap;

// FIXME: Split this up into multiple files: Collider, Route, Router, etc.

// FIXME: Implement the following:
// Collider<Route> for &str;
trait Collider<T = Self> {
    fn collides_with(&self, other: &T) -> bool;
}

macro_rules! comp_to_str {
    ($component:expr) => (
        match $component {
            &Component::Normal(ref comp) => {
                if let Some(string) = comp.to_str() { string }
                else { return true }
            },
            _ => return true
        };
    )
}

fn check_match_until(c: char, a: &str, b: &str, dir: bool) -> bool {
    let (a_len, b_len) = (a.len() as isize, b.len() as isize);
    let (mut i, mut j, delta) = if dir {
        (0, 0, 1)
    } else {
        (a_len - 1, b_len - 1, -1)
    };

    while i >= 0 && j >= 0 && i < a_len && j < b_len {
        let (c1, c2) = (a.char_at(i as usize), b.char_at(j as usize));
        if c1 == c || c2 == c {
            break;
        } else if c1 != c2 {
            return false;
        } else {
            i += delta;
            j += delta;
        }
    }

    return true;
}

impl<'a> Collider for Component<'a> {
    fn collides_with(&self, other: &Component<'a>) -> bool {
        let (a, b) = (comp_to_str!(self), comp_to_str!(other));
        check_match_until('<', a, b, true) && check_match_until('>', a, b, false)
    }
}

#[derive(Debug)]
struct Route {
    method: Method,
    mount: &'static str,
    route: &'static str,
    n_components: usize,
    path: PathBuf
}

pub struct Router {
    routes: Vec<Route> // for now, to check for collisions
}

impl Route {
    fn new(m: Method, mount: &'static str, route: &'static str) -> Route {
        let deduped_path = Route::dedup(mount, route);
        let path = PathBuf::from(deduped_path);

        Route {
            method: m,
            mount: mount,
            route: route,
            n_components: path.components().count(),
            path: path
        }
    }

    fn dedup(base: &'static str, route: &'static str) -> String {
        let mut deduped = String::with_capacity(base.len() + route.len() + 1);

        let mut last = '\0';
        for c in base.chars().chain("/".chars()).chain(route.chars()) {
            if !(last == '/' && c == '/') {
                deduped.push(c);
            }

            last = c;
        }

        deduped
    }
}

impl Collider for Route {
    fn collides_with(&self, b: &Route) -> bool {
        if self.n_components != b.n_components || self.method != b.method {
            return false;
        }

        let mut matches = 0;
        let mut a_components = self.path.components();
        let mut b_components = b.path.components();
        while let Some(ref c1) = a_components.next() {
            if let Some(ref c2) = b_components.next() {
                if c1.collides_with(c2) {
                    matches += 1;
                }
            }
        }

        // println!("Checked {:?} against {:?}: {}/{}", a, b, matches, n);
        matches == self.n_components
    }
}

// TODO: Are /hello and /hello/ the same? Or not? Currently, they're treated as
// such by Path, which is passed on to the collisions stuff.
impl Router {
    pub fn new() -> Router {
        Router {
            routes: Vec::new()
        }
    }

    // TODO: Use `method` argument
    pub fn add_route(&mut self, method: Method, base: &'static str,
                     route: &'static str) {
        let route = Route::new(method, base, route);
        println!("Mounted: {:?}", route);
        self.routes.push(route);
    }

    pub fn has_collisions(&self) -> bool {
        let mut map: HashMap<usize, Vec<&Route>> = HashMap::new();

        for route in &self.routes {
            let num_components = route.path.components().count();
            let mut list = if map.contains_key(&num_components) {
                map.get_mut(&num_components).unwrap()
            } else {
                map.insert(num_components, Vec::new());
                map.get_mut(&num_components).unwrap()
            };

            list.push(&route);
        }

        let mut result = false;
        for (_, routes) in map {
            for i in 0..routes.len() {
                for j in 0..routes.len() {
                    if i == j { continue }

                    let (a_route, b_route) = (&routes[i], &routes[j]);
                    if a_route.collides_with(b_route) {
                        result = true;
                        println!("{:?} and {:?} collide!", a_route, b_route);
                    }
                }
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::{Route, Collider};
    use Method;
    use Method::*;

    fn m_collide(a: (Method, &'static str), b: (Method, &'static str)) -> bool {
        let route_a = Route::new(a.0, "/", a.1);
        route_a.collides_with(&Route::new(b.0, "/", b.1))
    }

    fn collide(a: &'static str, b: &'static str) -> bool {
        let route_a = Route::new(Get, "/", a);
        route_a.collides_with(&Route::new(Get, "/", b))
    }

    #[test]
    fn simple_collisions() {
        assert!(collide("a", "a"));
        assert!(collide("/a", "/a"));
        assert!(collide("/hello", "/hello"));
        assert!(collide("/hello", "/hello/"));
        assert!(collide("/hello/there/how/ar", "/hello/there/how/ar"));
        assert!(collide("/hello/there", "/hello/there/"));
    }

    #[test]
    fn simple_param_collisions() {
        assert!(collide("/hello/<name>", "/hello/<person>"));
        assert!(collide("/hello/<name>/hi", "/hello/<person>/hi"));
        assert!(collide("/hello/<name>/hi/there", "/hello/<person>/hi/there"));
        assert!(collide("/<name>/hi/there", "/<person>/hi/there"));
        assert!(collide("/<name>/hi/there", "/dude/<name>/there"));
        assert!(collide("/<name>/<a>/<b>", "/<a>/<b>/<c>"));
        assert!(collide("/<name>/<a>/<b>/", "/<a>/<b>/<c>/"));
    }

    #[test]
    fn medium_param_collisions() {
        assert!(collide("/hello/<name>", "/hello/bob"));
        assert!(collide("/<name>", "//bob"));
    }

    #[test]
    fn hard_param_collisions() {
        assert!(collide("/<name>bob", "/<name>b"));
        assert!(collide("/a<b>c", "/abc"));
        assert!(collide("/a<b>c", "/azooc"));
        assert!(collide("/a<b>", "/a"));
        assert!(collide("/<b>", "/a"));
        assert!(collide("/<a>/<b>", "/a/b<c>"));
        assert!(collide("/<a>/bc<b>", "/a/b<c>"));
        assert!(collide("/<a>/bc<b>d", "/a/b<c>"));
    }

    #[test]
    fn non_collisions() {
        assert!(!collide("/a", "/b"));
        assert!(!collide("/a/b", "/a"));
        assert!(!collide("/a/b", "/a/c"));
        assert!(!collide("/a/hello", "/a/c"));
        assert!(!collide("/hello", "/a/c"));
        assert!(!collide("/hello/there", "/hello/there/guy"));
        assert!(!collide("/b<a>/there", "/hi/there"));
        assert!(!collide("/<a>/<b>c", "/hi/person"));
        assert!(!collide("/<a>/<b>cd", "/hi/<a>e"));
        assert!(!collide("/a<a>/<b>", "/b<b>/<a>"));
        assert!(!collide("/a/<b>", "/b/<b>"));
        assert!(!collide("/a<a>/<b>", "/b/<b>"));
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
}
