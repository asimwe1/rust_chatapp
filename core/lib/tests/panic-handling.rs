#[macro_use] extern crate rocket;

use rocket::Rocket;
use rocket::http::Status;
use rocket::local::blocking::Client;

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

fn rocket() -> Rocket {
    rocket::ignite()
        .mount("/", routes![panic_route])
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
    assert!(!response.into_string().unwrap().contains(":("));
}
