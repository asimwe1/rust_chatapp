extern crate rocket;

use rocket::{Rocket, Request, Response, Route};
use rocket::Method::*;

fn root(req: Request) -> Response<'static> {
    let name = req.get_param(0).unwrap_or("unnamed");
    Response::new(format!("Hello, {}!", name))
}

// TODO: Work with these lifetimes.
#[allow(dead_code)]
fn lifetime_root<'a>(req: Request<'a>) -> Response<'a> {
    Response::new(req.get_uri())
}

fn main() {
    let first = Route::new(Get, "/hello", root);
    let second = Route::new(Get, "/hello/<any>", root);
    Rocket::new("localhost", 8000).mount_and_launch("/", &[&first, &second]);

    // This below _should_ work.
    // let lifetime = Route::new(Get, "/other", lifetime_root);
    // Rocket::new("localhost", 8000).mount_and_launch("/", &[&lifetime]);
}
