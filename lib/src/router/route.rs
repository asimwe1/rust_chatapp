use term_painter::ToStyle;
use term_painter::Color::*;
use std::path::{Path, PathBuf};
use std::fmt;
use method::Method;
use super::Collider; // :D

// FIXME: Take in the handler! Or maybe keep that in `Router`?
#[derive(Debug)]
pub struct Route {
    method: Method,
    mount: &'static str,
    route: &'static str,
    n_components: usize,
    path: PathBuf
}

impl Route {
    pub fn new(m: Method, mount: &'static str, route: &'static str) -> Route {
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

    pub fn component_count(&self) -> usize {
        self.n_components
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

impl fmt::Display for Route {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {:?}", Green.paint(&self.method), Blue.paint(&self.path))
    }
}

impl Collider for Path {
    // FIXME: It's expensive to compute the number of components: O(n) per path
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

impl Collider for Route {
    fn collides_with(&self, b: &Route) -> bool {
        if self.n_components != b.n_components || self.method != b.method {
            return false;
        }

        self.path.collides_with(&b.path)
    }
}

impl<'a> Collider<Route> for &'a str {
    fn collides_with(&self, other: &Route) -> bool {
        let path = Path::new(self);
        path.collides_with(&other.path)
    }
}

impl Collider<str> for Route {
    fn collides_with(&self, other: &str) -> bool {
        other.collides_with(self)
    }
}
