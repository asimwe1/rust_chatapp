#[macro_use] extern crate rocket;

#[cfg(test)] mod tests;

use std::collections::HashMap;
use std::borrow::Cow;

use rocket::State;
use rocket::tokio::sync::Mutex;
use rocket_contrib::json::{Json, JsonValue, json};

use serde::{Serialize, Deserialize};

// The type to represent the ID of a message.
type Id = usize;

// We're going to store all of the messages here. No need for a DB.
type MessageMap<'r> = State<'r, Mutex<HashMap<Id, String>>>;

#[derive(Serialize, Deserialize)]
struct Message<'r> {
    id: Option<Id>,
    contents: Cow<'r, str>
}

#[post("/<id>", format = "json", data = "<message>")]
async fn new(id: Id, message: Json<Message<'_>>, map: MessageMap<'_>) -> JsonValue {
    let mut hashmap = map.lock().await;
    if hashmap.contains_key(&id) {
        json!({
            "status": "error",
            "reason": "ID exists. Try put."
        })
    } else {
        hashmap.insert(id, message.contents.to_string());
        json!({ "status": "ok" })
    }
}

#[put("/<id>", format = "json", data = "<message>")]
async fn update(id: Id, message: Json<Message<'_>>, map: MessageMap<'_>) -> Option<JsonValue> {
    let mut hashmap = map.lock().await;
    if hashmap.contains_key(&id) {
        hashmap.insert(id, message.contents.to_string());
        Some(json!({ "status": "ok" }))
    } else {
        None
    }
}

#[get("/<id>", format = "json")]
async fn get<'r>(id: Id, map: MessageMap<'r>) -> Option<Json<Message<'r>>> {
    let hashmap = map.lock().await;
    let contents = hashmap.get(&id)?.clone();
    Some(Json(Message {
        id: Some(id),
        contents: contents.into()
    }))
}

#[get("/echo", data = "<msg>")]
fn echo<'r>(msg: Json<Message<'r>>) -> Cow<'r, str> {
    msg.into_inner().contents
}

#[catch(404)]
fn not_found() -> JsonValue {
    json!({
        "status": "error",
        "reason": "Resource was not found."
    })
}

#[launch]
fn rocket() -> _ {
    rocket::ignite()
        .mount("/message", routes![new, update, get, echo])
        .register("/", catchers![not_found])
        .manage(Mutex::new(HashMap::<Id, String>::new()))
}
