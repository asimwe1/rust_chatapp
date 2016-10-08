extern crate rocket;

use rocket::{Request, Response, Route, Data};
use rocket::request::FromParam;
use rocket::http::Method::*;

fn forward(_req: &Request, data: Data) -> Response<'static> {
    Response::forward(data)
}

fn hi(_req: &Request, _: Data) -> Response<'static> {
    Response::new("Hello!")
}

fn name<'a>(req: &'a Request, _: Data) -> Response<'a> {
    Response::new(req.get_param(0).unwrap_or("unnamed"))
}

#[allow(dead_code)]
fn echo_url<'a>(req: &'a Request, _: Data) -> Response<'a> {
    let param = req.uri().as_str().split_at(6).1;
    Response::new(String::from_param(param))
}

fn main() {
    let always_forward = Route::ranked(1, Get, "/", forward);
    let hello = Route::ranked(2, Get, "/", hi);

    let echo = Route::new(Get, "/", echo_url);
    let name = Route::new(Get, "/<name>", name);

    rocket::ignite()
        .mount("/", vec![always_forward, hello])
        .mount("/hello", vec![name.clone()])
        .mount("/hi", vec![name])
        .mount("/echo:<str>", vec![echo])
        .launch();
}
