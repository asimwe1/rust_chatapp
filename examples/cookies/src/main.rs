#![feature(plugin, custom_derive, custom_attribute)]
#![plugin(rocket_macros)]

#[macro_use]
extern crate lazy_static;
extern crate rocket;
extern crate tera;

mod static_files;

use rocket::Rocket;
use rocket::response::{Cookied, Redirect};

lazy_static!(static ref TERA: tera::Tera = tera::Tera::new("templates/**/*"););

fn ctxt(message: Option<&str>) -> tera::Context {
    let mut context = tera::Context::new();
    context.add("have_message", &message.is_some());
    context.add("message", &message.unwrap_or("").to_string());
    context
}

#[derive(FromForm)]
struct Message {
    message: String
}

#[route(POST, path = "/submit", form = "<message>")]
fn submit(message: Message) -> Cookied<Redirect> {
    Cookied::new(Redirect::to("/")).add("message", &message.message)
}

#[route(GET, path = "/")]
fn index() -> tera::TeraResult<String> {
    TERA.render("index.html", ctxt(None))
}

fn main() {
    let mut rocket = Rocket::new("127.0.0.1", 8000);
    rocket.mount("/", static_files::routes());
    rocket.mount("/", routes![submit, index]);
    rocket.launch();
}
