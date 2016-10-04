#![feature(plugin, custom_derive, custom_attribute)]
#![plugin(rocket_codegen)]

#[macro_use]
extern crate lazy_static;
extern crate rocket;
extern crate tera;

use rocket::response::Redirect;
use rocket::http::{Cookie, Cookies};

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
fn submit(cookies: &Cookies, message: Message) -> Redirect {
    cookies.add(Cookie::new("message".into(), message.message));
    Redirect::to("/")
}

#[get("/")]
fn index(cookies: &Cookies) -> tera::TeraResult<String> {
    let message = cookies.find("message").map(|msg| msg.value);
    TERA.render("index.html", ctxt(message))
}

fn main() {
    let mut rocket = rocket::ignite();
    rocket.mount("/", routes![submit, index]);
    rocket.launch();
}
