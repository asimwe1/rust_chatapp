#![feature(plugin, rustc_macro)]
#![plugin(rocket_codegen)]

extern crate rocket_contrib;
extern crate rocket;
extern crate serde_json;
#[macro_use] extern crate serde_derive;

use rocket::{Rocket, Request, Error};
use rocket::response::Redirect;
use rocket_contrib::Template;

#[derive(Serialize)]
struct TemplateContext {
    name: String,
    items: Vec<String>
}

#[get("/")]
fn index() -> Redirect {
    Redirect::to("/hello/Unknown")
}

#[get("/hello/<name>")]
fn get(name: String) -> Template {
    let context = TemplateContext {
        name: name,
        items: vec!["One", "Two", "Three"].iter().map(|s| s.to_string()).collect()
    };

    Template::render("index", &context)
}

#[error(404)]
fn not_found<'r>(_: Error, req: &'r Request<'r>) -> Template {
    let mut map = std::collections::HashMap::new();
    map.insert("path", req.uri().as_str());
    Template::render("404", &map)
}

fn main() {
    let mut rocket = Rocket::new("localhost", 8000);
    rocket.catch(errors![not_found]);
    rocket.mount_and_launch("/", routes![index, get]);
}
