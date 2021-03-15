#[macro_use] extern crate rocket;

use rocket::{Rocket, Route, Request};
use rocket::data::Data;
use rocket::http::{Method, Status};
use rocket::local::blocking::Client;
use rocket::catcher::{Catcher, ErrorHandlerFuture};
use rocket::handler::HandlerFuture;

#[get("/panic")]
fn panic_route() -> &'static str {
    panic!("Panic in route")
}

#[catch(404)]
fn panic_catcher() -> &'static str {
    panic!("Panic in catcher")
}

#[catch(500)]
fn ise() -> &'static str {
    "Hey, sorry! :("
}

#[catch(500)]
fn double_panic() {
    panic!("so, so sorry...")
}

fn pre_future_route<'r>(_: &'r Request<'_>, _: Data) -> HandlerFuture<'r> {
    panic!("hey now...");
}

fn pre_future_catcher<'r>(_: Status, _: &'r Request) -> ErrorHandlerFuture<'r> {
    panic!("a panicking pre-future catcher")
}

fn rocket() -> Rocket {
    let pre_future_panic = Route::new(Method::Get, "/pre", pre_future_route);
    rocket::ignite()
        .mount("/", routes![panic_route])
        .mount("/", vec![pre_future_panic])
        .register(catchers![panic_catcher, ise])
}

#[test]
fn catches_route_panic() {
    let client = Client::debug(rocket()).unwrap();
    let response = client.get("/panic").dispatch();
    assert_eq!(response.status(), Status::InternalServerError);
    assert_eq!(response.into_string().unwrap(), "Hey, sorry! :(");
}

#[test]
fn catches_catcher_panic() {
    let client = Client::debug(rocket()).unwrap();
    let response = client.get("/noroute").dispatch();
    assert_eq!(response.status(), Status::InternalServerError);
    assert_eq!(response.into_string().unwrap(), "Hey, sorry! :(");
}

#[test]
fn catches_double_panic() {
    let rocket = rocket().register(catchers![double_panic]);
    let client = Client::debug(rocket).unwrap();
    let response = client.get("/noroute").dispatch();
    assert_eq!(response.status(), Status::InternalServerError);
    assert!(response.into_string().unwrap().contains("Rocket"));
}

#[test]
fn catches_early_route_panic() {
    let client = Client::debug(rocket()).unwrap();
    let response = client.get("/pre").dispatch();
    assert_eq!(response.status(), Status::InternalServerError);
    assert_eq!(response.into_string().unwrap(), "Hey, sorry! :(");
}

#[test]
fn catches_early_catcher_panic() {
    let panic_catcher = Catcher::new(404, pre_future_catcher);

    let client = Client::debug(rocket().register(vec![panic_catcher])).unwrap();
    let response = client.get("/idontexist").dispatch();
    assert_eq!(response.status(), Status::InternalServerError);
    assert_eq!(response.into_string().unwrap(), "Hey, sorry! :(");
}
