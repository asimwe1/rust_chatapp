extern crate rocket;

use rocket::{Request, Response, Route};
use rocket::http::Method::*;

fn root(req: &Request) -> Response<'static> {
    let name = req.get_param(0).unwrap_or("unnamed");
    Response::new(format!("Hello, {}!", name))
}

fn name<'a>(req: &'a Request) -> Response<'a> {
    Response::new(req.get_param(0).unwrap_or("unnamed"))
}

#[allow(dead_code)]
fn echo_url<'a>(req: &'a Request) -> Response<'a> {
    Response::new(req.uri().as_str().split_at(6).1)
}

fn main() {
    let first = Route::new(Get, "/hello", root);
    let second = Route::new(Get, "/hello/<any>", root);
    let name = Route::new(Get, "/<name>", name);
    let echo = Route::new(Get, "/", echo_url);

    rocket::ignite()
        .mount("/", vec![first, second, name])
        .mount("/echo:<str>", vec![echo])
        .launch();
}
