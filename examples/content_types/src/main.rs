#[macro_use] extern crate rocket;

#[cfg(test)] mod tests;

use std::io;

use rocket::request::Request;
use rocket::data::{Data, ToByteUnit};
use rocket::response::content::{Json, Html};

use serde::{Serialize, Deserialize};

// NOTE: This example explicitly uses the `Json` type from `response::content`
// for demonstration purposes. In a real application, _always_ prefer to use
// `rocket_contrib::json::Json` instead!

#[derive(Debug, Serialize, Deserialize)]
struct Person {
    name: String,
    age: u8,
}

// In a `GET` request and all other non-payload supporting request types, the
// preferred media type in the Accept header is matched against the `format` in
// the route attribute. Note: if this was a real application, we'd use
// `rocket_contrib`'s built-in JSON support and return a `JsonValue` instead.
#[get("/<name>/<age>", format = "json")]
fn get_hello(name: String, age: u8) -> io::Result<Json<String>> {
    // NOTE: In a real application, we'd use `rocket_contrib::json::Json`.
    let person = Person { name, age };
    Ok(Json(serde_json::to_string(&person)?))
}

// In a `POST` request and all other payload supporting request types, the
// content type is matched against the `format` in the route attribute.
//
// Note that `content::Json` simply sets the content-type to `application/json`.
// In a real application, we wouldn't use `serde_json` directly; instead, we'd
// use `contrib::Json` to automatically serialize a type into JSON.
#[post("/<age>", format = "plain", data = "<name_data>")]
async fn post_hello(age: u8, name_data: Data) -> io::Result<Json<String>> {
    let name = name_data.open(64.bytes()).into_string().await?;
    let person = Person { name: name.into_inner(), age };
    // NOTE: In a real application, we'd use `rocket_contrib::json::Json`.
    Ok(Json(serde_json::to_string(&person)?))
}

#[catch(404)]
fn not_found(request: &Request<'_>) -> Html<String> {
    let html = match request.format() {
        Some(ref mt) if !mt.is_json() && !mt.is_plain() => {
            format!("<p>'{}' requests are not supported.</p>", mt)
        }
        _ => format!("<p>Sorry, '{}' is an invalid path! Try \
                 /hello/&lt;name&gt;/&lt;age&gt; instead.</p>",
                 request.uri())
    };

    Html(html)
}

#[launch]
fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .mount("/hello", routes![get_hello, post_hello])
        .register("/", catchers![not_found])
}
