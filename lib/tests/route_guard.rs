#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]

extern crate rocket;

use std::path::PathBuf;
use rocket::Route;

#[get("/<path..>")]
fn files(route: &Route, path: PathBuf) -> String {
    format!("{}/{}", route.base.path(), path.to_string_lossy())
}

mod route_guard_tests {
    use super::*;

    use rocket::Rocket;
    use rocket::testing::MockRequest;
    use rocket::http::Method::*;

    fn assert_path(rocket: &Rocket, path: &str) {
        let mut req = MockRequest::new(Get, path);
        let mut res = req.dispatch_with(&rocket);
        assert_eq!(res.body_string(), Some(path.into()));
    }

    #[test]
    fn check_mount_path() {
        let rocket = rocket::ignite()
            .mount("/first", routes![files])
            .mount("/second", routes![files]);

        assert_path(&rocket, "/first/some/path");
        assert_path(&rocket, "/second/some/path");
        assert_path(&rocket, "/first/second/b/c");
        assert_path(&rocket, "/second/a/b/c");
    }
}
