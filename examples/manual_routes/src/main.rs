extern crate rocket;

use std::io;
use std::fs::File;

use rocket::{Request, Response, Route, Data};
use rocket::http::StatusCode;
use rocket::request::FromParam;
use rocket::http::Method::*;

fn forward(_req: &Request, data: Data) -> Response<'static> {
    Response::forward(data)
}

fn hi(_req: &Request, _: Data) -> Response<'static> {
    Response::complete("Hello!")
}

fn name<'a>(req: &'a Request, _: Data) -> Response<'a> {
    Response::complete(req.get_param(0).unwrap_or("unnamed"))
}

fn echo_url<'a>(req: &'a Request, _: Data) -> Response<'a> {
    let param = req.uri().as_str().split_at(6).1;
    Response::complete(String::from_param(param))
}

fn upload(req: &Request, data: Data) -> Response {
    if !req.content_type().is_text() {
        return Response::failed(StatusCode::BadRequest);
    }

    let file = File::create("/tmp/upload.txt");
    if let Ok(mut file) = file {
        if let Ok(n) = io::copy(&mut data.open(), &mut file) {
            return Response::complete(format!("OK: {} bytes uploaded.", n));
        }

        println!("    => Failed copying.");
        Response::failed(StatusCode::InternalServerError)
    } else {
        println!("    => Couldn't open file: {:?}", file.unwrap_err());
        Response::failed(StatusCode::InternalServerError)
    }
}

fn main() {
    let always_forward = Route::ranked(1, Get, "/", forward);
    let hello = Route::ranked(2, Get, "/", hi);

    let echo = Route::new(Get, "/", echo_url);
    let name = Route::new(Get, "/<name>", name);
    let upload_route = Route::new(Post, "/upload", upload);

    rocket::ignite()
        .mount("/", vec![always_forward, hello, upload_route])
        .mount("/hello", vec![name.clone()])
        .mount("/hi", vec![name])
        .mount("/echo:<str>", vec![echo])
        .launch();
}
