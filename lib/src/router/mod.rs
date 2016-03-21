mod collider;
mod route;

pub use self::collider::Collider;

use self::route::Route;
use std::collections::hash_map::HashMap;
use method::Method;

pub struct Router {
    routes: Vec<Route> // for now, to check for collisions
}

impl Router {
    pub fn new() -> Router {
        Router {
            routes: Vec::new()
        }
    }

    pub fn add_route(&mut self, method: Method, base: &'static str,
                     route: &'static str) {
        let route = Route::new(method, base, route);
        self.routes.push(route);
    }

    pub fn has_collisions(&self) -> bool {
        let mut map: HashMap<usize, Vec<&Route>> = HashMap::new();

        for route in &self.routes {
            let num_components = route.component_count();
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

