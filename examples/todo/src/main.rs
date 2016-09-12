#![feature(plugin, custom_derive, custom_attribute)]
#![plugin(rocket_codegen, serde_macros, diesel_codegen)]

extern crate rocket;
extern crate tera;
#[macro_use] extern crate diesel;
#[macro_use] extern crate lazy_static;
extern crate serde_json;

mod static_files;
mod task;

use rocket::Rocket;
use rocket::response::{Flash, Redirect};
use task::Task;

lazy_static!(static ref TERA: tera::Tera = tera::Tera::new("static/*.html"););

fn ctxt(msg: Option<(&str, &str)>) -> tera::Context {
    let unwrapped_msg = msg.unwrap_or(("", ""));
    let mut context = tera::Context::new();
    context.add("has_msg", &msg.is_some());
    context.add("msg_type", &unwrapped_msg.0.to_string());
    context.add("msg", &unwrapped_msg.1.to_string());
    context.add("tasks", &Task::all());
    context
}

#[post("/", form = "<todo>")]
fn new(todo: Task) -> Result<Flash<Redirect>, tera::TeraResult<String>> {
    if todo.description.is_empty() {
        let context = ctxt(Some(("error", "Description cannot be empty.")));
        Err(TERA.render("index.html", context))
    } else if todo.insert() {
        Ok(Flash::success(Redirect::to("/"), "Todo successfully added."))
    } else {
        let context = ctxt(Some(("error", "Whoops! The server failed.")));
        Err(TERA.render("index.html", context))
    }
}

// Should likely do something to simulate PUT.
#[get("/<id>/toggle")]
fn toggle(id: i32) -> Result<Redirect, tera::TeraResult<String>> {
    if Task::toggle_with_id(id) {
        Ok(Redirect::to("/"))
    } else {
        let context = ctxt(Some(("error", "Could not toggle that task.")));
        Err(TERA.render("index.html", context))
    }
}

// Should likely do something to simulate DELETE.
#[get("/<id>/delete")]
fn delete(id: i32) -> Result<Flash<Redirect>, tera::TeraResult<String>> {
    if Task::delete_with_id(id) {
        Ok(Flash::success(Redirect::to("/"), "Todo was deleted."))
    } else {
        let context = ctxt(Some(("error", "Could not delete that task.")));
        Err(TERA.render("index.html", context))
    }
}

#[get("/")]
fn index(msg: Option<Flash<()>>) -> tera::TeraResult<String> {
    TERA.render("index.html", ctxt(msg.as_ref().map(|m| (m.name(), m.msg()))))
}

fn main() {
    let mut rocket = Rocket::new("127.0.0.1", 8000);
    rocket.mount("/", routes![index, static_files::all])
          .mount("/todo/", routes![new, delete, toggle]);
    rocket.launch();
}
