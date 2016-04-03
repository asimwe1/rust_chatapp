extern crate rocket;

use rocket::{Rocket, Request, Response, Route};
use rocket::Method::*;

fn root(req: Request) -> Response {
    let name = req.get_param(0).unwrap_or("unnamed");
    Response::new(format!("Hello, {}!", name))
}

#[allow(dead_code)]
fn echo_url<'a>(req: Request<'a>) -> Response<'a> {
    Response::new(req.get_uri().split_at(6).1)
}

fn main() {
    let mut rocket = Rocket::new("localhost", 8000);

    let first = Route::new(Get, "/hello", root);
    let second = Route::new(Get, "/hello/<any>", root);
    rocket.mount("/", vec![first, second]);

    let echo = Route::new(Get, "/", echo_url);
    rocket.mount("/echo:<str>", vec![echo]);

    rocket.launch();
}
