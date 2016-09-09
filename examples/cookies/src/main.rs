#![feature(plugin, custom_derive, custom_attribute)]
#![plugin(rocket_codegen)]

#[macro_use]
extern crate lazy_static;
extern crate rocket;
extern crate tera;

use rocket::Rocket;
use rocket::response::{Cookied, Redirect};
use rocket::request::Cookies;

lazy_static!(static ref TERA: tera::Tera = tera::Tera::new("templates/**/*"););

fn ctxt(message: Option<String>) -> tera::Context {
    let mut context = tera::Context::new();
    context.add("have_message", &message.is_some());
    context.add("message", &message.unwrap_or("".to_string()));
    context
}

#[derive(FromForm)]
struct Message {
    message: String
}

#[post("/submit", form = "<message>")]
fn submit(message: Message) -> Cookied<Redirect> {
    Cookied::new(Redirect::to("/")).add("message", &message.message)
}

#[get("/")]
fn index(cookies: Cookies) -> tera::TeraResult<String> {
    let message = cookies.find("message").map(|msg| msg.value);
    TERA.render("index.html", ctxt(message))
}

fn main() {
    let mut rocket = Rocket::new("127.0.0.1", 8000);
    rocket.mount("/", routes![submit, index]);
    rocket.launch();
}
