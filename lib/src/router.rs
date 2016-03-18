use method::Method;
use std::path::{Path, PathBuf, Component};
use std::collections::hash_map::HashMap;

pub struct Router {
    paths: Vec<PathBuf> // for now, to check for collisions
}

fn is_variadic(c: &Component) -> bool {
    match c {
        &Component::Normal(ref comp) => comp.to_str().unwrap().starts_with("<"),
        _ => false
    }
}

#[inline]
fn routes_collide(n: usize, a: &Path, b: &Path) -> bool {
    let mut matches = 0;

    let mut a_components = a.components();
    let mut b_components = b.components();

    while let Some(ref c1) = a_components.next() {
        if let Some(ref c2) = b_components.next() {
            if c1 == c2 {
                matches += 1;
            } else if is_variadic(c1) || is_variadic(c2) {
                matches += 1;
            }
        }
    }

    // println!("Checked {:?} against {:?}: {}/{}", a, b, matches, n);
    matches == n
}

// TODO: Are /hello and /hello/ the same? Or not? Currently, they're treated as
// such by Path, which is passed on to the collisions stuff.
impl Router {
    pub fn new() -> Router {
        Router {
            paths: Vec::new()
        }
    }

    // TODO: Use `method` argument
    pub fn add_route(&mut self, _method: Method, base: &str, route: &str) {
        // Allocate enough space for the worst case.
        let mut deduped = String::with_capacity(base.len() + route.len() + 1);

        let mut last = '\0';
        for c in base.chars().chain("/".chars()).chain(route.chars()) {
            if !(last == '/' && c == '/') {
                deduped.push(c);
            }

            last = c;
        }

        let path = PathBuf::from(deduped);
        println!("Mounted: {:?}", path);
        self.paths.push(path);
    }

    pub fn has_collisions(&self) -> bool {
        let mut map: HashMap<usize, Vec<&Path>> = HashMap::new();

        for route in &self.paths {
            let num_components = route.components().count();
            println!("Found {} for {:?}", num_components, route);

            let mut list = if map.contains_key(&num_components) {
                map.get_mut(&num_components).unwrap()
            } else {
                map.insert(num_components, Vec::new());
                map.get_mut(&num_components).unwrap()
            };

            list.push(route.as_path());
        }

        let mut result = false;
        for (num_components, routes) in map.iter() {
            for i in 0..routes.len() {
                for j in 0..routes.len() {
                    if i == j { continue }

                    let a_route = &routes[i];
                    let b_route = &routes[j];
                    if routes_collide(*num_components, a_route, b_route) {
                        result = true;
                        println!("{:?} and {:?} collide!", a_route, b_route);
                    }
                }
            }
        }

        result
    }
}
