extern crate rocket;

use rocket::{Request, Response, Route};
use rocket::http::Method::*;

fn root<'r>(req: &'r Request<'r>) -> Response<'r> {
    let name = req.get_param(0).unwrap_or("unnamed");
    Response::new(format!("Hello, {}!", name))
}

#[allow(dead_code)]
fn echo_url<'a>(req: &'a Request<'a>) -> Response<'a> {
    Response::new(req.uri().as_str().split_at(6).1)
}

fn main() {
    let mut rocket = rocket::ignite();

    let first = Route::new(Get, "/hello", root);
    let second = Route::new(Get, "/hello/<any>", root);
    rocket.mount("/", vec![first, second]);

    let echo = Route::new(Get, "/", echo_url);
    rocket.mount("/echo:<str>", vec![echo]);

    rocket.launch();
}
