use term_painter::ToStyle;
use term_painter::Color::*;
use std::path::{Path, PathBuf};
use std::fmt;
use method::Method;
use super::Collider; // :D
use std::path::Component;
use Handler;

// FIXME: Take in the handler! Or maybe keep that in `Router`?
pub struct Route {
    method: Method,
    n_components: usize,
    pub handler: Handler<'static>,
    path: PathBuf
}

macro_rules! comp_to_str {
    ($component:expr) => (
        match $component {
            &Component::Normal(ref comp) => {
                if let Some(string) = comp.to_str() { string }
                else { panic!("Whoops, no string!") }
            },
            &Component::RootDir => "/",
            &c@_ =>  panic!("Whoops, not normal: {:?}!", c)
        };
    )
}

impl Route {
    pub fn new(m: Method, mount: &'static str, route: &'static str,
               handler: Handler<'static>) -> Route {
        let deduped_path = Route::dedup(mount, route);
        let path = PathBuf::from(deduped_path);

        Route {
            method: m,
            n_components: path.components().count(),
            handler: handler,
            path: path,
        }
    }

    #[inline]
    pub fn component_count(&self) -> usize {
        self.n_components
    }

    // FIXME: This is dirty (the comp_to_str and the RootDir thing). Might need
    // to have my own wrapper arround path strings.
    // FIXME: Decide whether a component has to be fully variable or not. That
    // is, whether you can have: /a<a>b/
    // TODO: Don't return a Vec...take in an &mut [&'a str] (no alloc!)
    pub fn get_params<'a>(&self, uri: &'a str) -> Vec<&'a str> {
        let mut result = Vec::with_capacity(self.component_count());
        let route_components = self.path.components();
        let uri_components = Path::new(uri).components();

        for (route_comp, uri_comp) in route_components.zip(uri_components) {
            if comp_to_str!(&route_comp).starts_with("<") {
                result.push(comp_to_str!(&uri_comp));
            }
        }

        result
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
