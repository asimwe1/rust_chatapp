#![feature(plugin, custom_derive)]
#![plugin(rocket_macros, serde_macros)]

extern crate rocket;
extern crate serde_json;

use rocket::{Rocket, Request, ContentType, Error};

#[derive(Debug, Serialize, Deserialize)]
struct Person {
    name: String,
    age: i8,
}

#[GET(path = "/<name>/<age>", content = "application/json")]
fn hello(name: String, age: i8) -> String {
    let person = Person {
        name: name,
        age: age,
    };

    serde_json::to_string(&person).unwrap()
}

#[error(code = "404")]
fn not_found<'r>(error: Error, request: &'r Request<'r>) -> String {
    match error {
        Error::NoRoute if !request.content_type().is_json() => {
            format!("<p>This server only supports JSON requests, not '{}'.</p>",
                    request.content_type())
        }
        Error::NoRoute => {
            format!("<p>Sorry, this server but '{}' is not a valid path!</p>
                    <p>Try visiting /hello/&lt;name&gt;/&lt;age&gt; instead.</p>",
                    request.uri())
        }
        _ => format!("<p>Bad Request</p>"),
    }
}

fn main() {
    let mut rocket = Rocket::new("0.0.0.0", 8000);
    rocket.mount("/hello", routes![hello]);
    rocket.catch(errors![not_found]);
    rocket.launch();
}
