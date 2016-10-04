#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen, serde_macros)]

extern crate rocket;
extern crate serde_json;

use rocket::{Rocket, Request, Error};
use rocket::http::ContentType;
use rocket::response::data;

#[derive(Debug, Serialize, Deserialize)]
struct Person {
    name: String,
    age: i8,
}

#[get("/<name>/<age>", format = "application/json")]
fn hello(content_type: ContentType, name: String, age: i8) -> data::JSON<String> {
    let person = Person {
        name: name,
        age: age,
    };

    println!("ContentType: {}", content_type);
    data::JSON(serde_json::to_string(&person).unwrap())
}

#[error(404)]
fn not_found<'r>(error: Error, request: &'r Request<'r>) -> String {
    match error {
        Error::BadMethod if !request.content_type().is_json() => {
            format!("<p>This server only supports JSON requests, not '{}'.</p>",
                    request.content_type())
        }
        Error::NoRoute => {
            format!("<p>Sorry, '{}' is not a valid path!</p>
                    <p>Try visiting /hello/&lt;name&gt;/&lt;age&gt; instead.</p>",
                    request.uri())
        }
        _ => format!("<p>Bad Request</p>"),
    }
}

fn main() {
    let mut rocket = Rocket::new("0.0.0.0", 8000);
    rocket.mount("/hello", routes![hello]).catch(errors![not_found]);
    rocket.launch();
}
