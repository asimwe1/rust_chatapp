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
use rocket::response::Redirect;
use task::Task;

lazy_static!(static ref TERA: tera::Tera = tera::Tera::new("templates/**/*"););

fn ctxt(error: Option<&str>) -> tera::Context {
    let mut context = tera::Context::new();
    context.add("error", &error.is_some());
    context.add("msg", &error.unwrap_or("").to_string());
    context.add("tasks", &Task::all());
    context
}

#[post("/", form = "<todo>")]
fn new(todo: Task) -> Result<Redirect, tera::TeraResult<String>> {
    if todo.description.is_empty() {
        let context = ctxt(Some("Description cannot be empty."));
        Err(TERA.render("index.html", context))
    } else if todo.insert() {
        Ok(Redirect::to("/")) // Say that it was added...somehow.
    } else {
        let context = ctxt(Some("Whoops! The server failed."));
        Err(TERA.render("index.html", context))
    }
}

// Should likely do something to simulate PUT.
#[get("/<id>/toggle")]
fn toggle(id: i32) -> Result<Redirect, tera::TeraResult<String>> {
    if Task::toggle_with_id(id) {
        Ok(Redirect::to("/")) // Say that it was added...somehow.
    } else {
        let context = ctxt(Some("Could not toggle that task."));
        Err(TERA.render("index.html", context))
    }
}

// Should likely do something to simulate DELETE.
#[get("/<id>/delete")]
fn delete(id: i32) -> Result<Redirect, tera::TeraResult<String>> {
    if Task::delete_with_id(id) {
        Ok(Redirect::to("/")) // Say that it was added...somehow.
    } else {
        let context = ctxt(Some("Could not delete that task."));
        Err(TERA.render("index.html", context))
    }
}

#[get("/")]
fn index() -> tera::TeraResult<String> {
    TERA.render("index.html", ctxt(None))
}

fn main() {
    let mut rocket = Rocket::new("127.0.0.1", 8000);
    rocket.mount("/", routes![index, static_files::all, static_files::all_level_one]);
    rocket.mount("/todo/", routes![new, delete, toggle]);
    rocket.launch();
}
