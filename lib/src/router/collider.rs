use std::path::Component;
use std::path::Path;

pub trait Collider<T: ?Sized = Self> {
    fn collides_with(&self, other: &T) -> bool;
}

pub fn index_match_until(break_c: char, a: &str, b: &str, dir: bool)
        -> Option<(isize, isize)> {
    let (a_len, b_len) = (a.len() as isize, b.len() as isize);
    let (mut i, mut j, delta) = if dir {
        (0, 0, 1)
    } else {
        (a_len - 1, b_len - 1, -1)
    };

    while i >= 0 && j >= 0 && i < a_len && j < b_len {
        let (c1, c2) = (a.char_at(i as usize), b.char_at(j as usize));
        if c1 == break_c || c2 == break_c {
            break;
        } else if c1 != c2 {
            return None;
        } else {
            i += delta;
            j += delta;
        }
    }

    return Some((i, j));
}

fn do_match_until(break_c: char, a: &str, b: &str, dir: bool) -> bool {
    index_match_until(break_c, a, b, dir).is_some()
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

impl Collider for Path {
    // TODO: It's expensive to compute the number of components: O(n) per path
    // where n == number of chars.
    //
    // Idea: Create a `CachedPath` type that caches the number of components
    // similar to the way `Route` does it.
    fn collides_with(&self, b: &Path) -> bool {
        if self.components().count() != b.components().count() {
            return false;
        }

        let mut a_components = self.components();
        let mut b_components = b.components();
        while let Some(ref c1) = a_components.next() {
            if let Some(ref c2) = b_components.next() {
                if !c1.collides_with(c2) {
                    return false
                }
            }
        }

        true
    }
}

impl<'a> Collider for Component<'a> {
    fn collides_with(&self, other: &Component<'a>) -> bool {
        let (a, b) = (comp_to_str!(self), comp_to_str!(other));
        do_match_until('<', a, b, true) && do_match_until('>', a, b, false)
    }
}

impl<'a> Collider<str> for &'a str {
    fn collides_with(&self, other: &str) -> bool {
        Path::new(self).collides_with(Path::new(other))
    }
}

#[cfg(test)]
mod tests {
    use router::Collider;
    use router::route::Route;
    use Method;
    use Method::*;
    use {Request, Response};

    type SimpleRoute = (Method, &'static str);

    fn dummy_handler(_req: Request) -> Response<'static> {
        Response::empty()
    }

    fn m_collide(a: SimpleRoute, b: SimpleRoute) -> bool {
        let route_a = Route::new(a.0, "/", a.1, dummy_handler);
        route_a.collides_with(&Route::new(b.0, "/", b.1, dummy_handler))
    }

    fn collide(a: &'static str, b: &'static str) -> bool {
        let route_a = Route::new(Get, "/", a, dummy_handler);
        route_a.collides_with(&Route::new(Get, "/", b, dummy_handler))
    }

    fn s_r_collide(a: &'static str, b: &'static str) -> bool {
        a.collides_with(&Route::new(Get, "/", b, dummy_handler))
    }

    fn r_s_collide(a: &'static str, b: &'static str) -> bool {
        let route_a = Route::new(Get, "/", a, dummy_handler);
        route_a.collides_with(b)
    }

    fn s_s_collide(a: &'static str, b: &'static str) -> bool {
        a.collides_with(b)
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

    #[test]
    fn test_str_non_collisions() {
        assert!(!s_r_collide("/a", "/b"));
        assert!(!s_r_collide("/a/b", "/a"));
        assert!(!s_r_collide("/a/b", "/a/c"));
        assert!(!s_r_collide("/a/hello", "/a/c"));
        assert!(!s_r_collide("/hello", "/a/c"));
        assert!(!s_r_collide("/hello/there", "/hello/there/guy"));
        assert!(!s_r_collide("/b<a>/there", "/hi/there"));
        assert!(!s_r_collide("/<a>/<b>c", "/hi/person"));
        assert!(!s_r_collide("/<a>/<b>cd", "/hi/<a>e"));
        assert!(!s_r_collide("/a<a>/<b>", "/b<b>/<a>"));
        assert!(!s_r_collide("/a/<b>", "/b/<b>"));
        assert!(!s_r_collide("/a<a>/<b>", "/b/<b>"));
        assert!(!r_s_collide("/a", "/b"));
        assert!(!r_s_collide("/a/b", "/a"));
        assert!(!r_s_collide("/a/b", "/a/c"));
        assert!(!r_s_collide("/a/hello", "/a/c"));
        assert!(!r_s_collide("/hello", "/a/c"));
        assert!(!r_s_collide("/hello/there", "/hello/there/guy"));
        assert!(!r_s_collide("/b<a>/there", "/hi/there"));
        assert!(!r_s_collide("/<a>/<b>c", "/hi/person"));
        assert!(!r_s_collide("/<a>/<b>cd", "/hi/<a>e"));
        assert!(!r_s_collide("/a<a>/<b>", "/b<b>/<a>"));
        assert!(!r_s_collide("/a/<b>", "/b/<b>"));
        assert!(!r_s_collide("/a<a>/<b>", "/b/<b>"));
    }

    #[test]
    fn test_str_collisions() {
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
    }
}
