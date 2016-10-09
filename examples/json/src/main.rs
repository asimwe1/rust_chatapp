#![feature(plugin, proc_macro)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate serde_json;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;

use rocket_contrib::JSON;
use std::collections::HashMap;
use std::sync::Mutex;

// The type to represent the ID of a message.
type ID = usize;
type SimpleMap = HashMap<&'static str, &'static str>;

// We're going to store all of the messages here. No need for a DB.
lazy_static! {
    static ref MAP: Mutex<HashMap<ID, String>> = Mutex::new(HashMap::new());
}

#[derive(Serialize, Deserialize)]
struct Message {
    id: Option<ID>,
    contents: String
}

// TODO: This example can be improved by using `route` with muliple HTTP verbs.
// To be precise, put/post could/should look like:
// #[route(PUT, POST, path = "/<id>", format = "application/json")]
// fn f(method: Method, id: ID, message: JSON<Message>) -> Option<JSON<SimpleMap>> {
//     let mut hashmap = MAP.lock().unwrap();
//     let exists = hashmap.contains_key(&id);
//     if method == Method::Put && exists || method == Method::Post && !exists {
//         hashmap.insert(id, message.0.contents);
//         return Ok(JSON(map!{ "status" => "ok" }))
//     }
//
//     None
// }

#[post("/<id>", format = "application/json")]
fn new(id: ID, message: JSON<Message>) -> JSON<SimpleMap> {
    let mut hashmap = MAP.lock().unwrap();
    if hashmap.contains_key(&id) {
        JSON(map!{
            "status" => "error",
            "reason" => "ID exists. Try put."
        })
    } else {
        hashmap.insert(id, message.0.contents);
        JSON(map!{ "status" => "ok" })
    }
}

#[put("/<id>", format = "application/json")]
fn update(id: ID, message: JSON<Message>) -> Option<JSON<SimpleMap>> {
    let mut hashmap = MAP.lock().unwrap();
    if hashmap.contains_key(&id) {
        hashmap.insert(id, message.0.contents);
        Some(JSON(map!{ "status" => "ok" }))
    } else {
        None
    }
}

#[get("/<id>", format = "application/json")]
fn get(id: ID) -> Option<JSON<Message>> {
    let hashmap = MAP.lock().unwrap();
    hashmap.get(&id).map(|contents| {
        JSON(Message {
            id: Some(id),
            contents: contents.clone()
        })
    })
}

#[error(404)]
fn not_found() -> JSON<SimpleMap> {
    JSON(map! {
        "status" => "error",
        "reason" => "Resource was not found."
    })
}

fn main() {
    rocket::ignite()
        .mount("/message", routes![new, update, get])
        .catch(errors![not_found])
        .launch();
}
