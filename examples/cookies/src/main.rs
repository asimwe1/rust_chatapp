#![feature(plugin, custom_derive, custom_attribute)]
#![plugin(rocket_codegen)]

#[macro_use]
extern crate lazy_static;
extern crate rocket_contrib;
extern crate rocket;

use std::collections::HashMap;

use rocket::response::Redirect;
use rocket::http::{Cookie, Cookies};
use rocket_contrib::Template;

#[derive(FromForm)]
struct Message {
    message: String
}

#[post("/submit", form = "<message>")]
fn submit(cookies: &Cookies, message: Message) -> Redirect {
    cookies.add(Cookie::new("message".into(), message.message));
    Redirect::to("/")
}

#[get("/")]
fn index(cookies: &Cookies) -> Template {
    let mut context = HashMap::new();
    if let Some(msg) = cookies.find("message").map(|msg| msg.value) {
        context.insert("message", msg);
    }

    Template::render("index", &context)
}

fn main() {
    rocket::ignite().mount("/", routes![submit, index]).launch()
}
