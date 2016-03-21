mod collider;
mod route;

pub use self::collider::Collider;

use term_painter::ToStyle;
use term_painter::Color::*;
use self::route::Route;
use std::collections::hash_map::HashMap;
use std::path::Path;
use method::Method;

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
                     route: &'static str) {
        let route = Route::new(method, base, route);
        let selector = (method, route.component_count());
        self.routes.entry(selector).or_insert(vec![]).push(route);
    }

    // TODO: Make a `Router` trait with this function. Rename this `Router`
    // struct to something like `RocketRouter`.
    // TODO: Return an array of matches to the parameters.
    pub fn route<'a>(&self, method: Method, uri: &'a str) {
        let path = Path::new(uri);
        let num_components = path.components().count();
        if let Some(routes) = self.routes.get(&(method, num_components)) {
            for route in routes {
                if route.collides_with(uri) {
                    println!("Matched {} to: {}", uri, route);
                }
            }
        }
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

