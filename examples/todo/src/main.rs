#![feature(plugin, custom_derive, custom_attribute, proc_macro)]
#![plugin(rocket_codegen, diesel_codegen)]

extern crate rocket;
extern crate serde_json;
#[macro_use] extern crate diesel;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;

mod static_files;
mod task;

use rocket::response::{Flash, Redirect};
use rocket_contrib::Template;
use task::Task;

#[derive(Debug, Serialize)]
struct Context<'a, 'b>{msg: Option<(&'a str, &'b str)>, tasks: Vec<Task>}

impl<'a, 'b> Context<'a, 'b> {
    pub fn err(msg: &'a str) -> Context<'static, 'a> {
        Context{msg: Some(("error", msg)), tasks: Task::all()}
    }

    pub fn raw(msg: Option<(&'a str, &'b str)>) -> Context<'a, 'b> {
        Context{msg: msg, tasks: Task::all()}
    }
}

#[post("/", form = "<todo>")]
fn new(todo: Task) -> Result<Flash<Redirect>, Template> {
    if todo.description.is_empty() {
        Err(Template::render("index", &Context::err("Description cannot be empty.")))
    } else if todo.insert() {
        Ok(Flash::success(Redirect::to("/"), "Todo successfully added."))
    } else {
        Err(Template::render("index", &Context::err("Whoops! The server failed.")))
    }
}

// Should likely do something to simulate PUT.
#[put("/<id>")]
fn toggle(id: i32) -> Result<Redirect, Template> {
    if Task::toggle_with_id(id) {
        Ok(Redirect::to("/"))
    } else {
        Err(Template::render("index", &Context::err("Couldn't toggle task.")))
    }
}

// Should likely do something to simulate DELETE.
#[delete("/<id>")]
fn delete(id: i32) -> Result<Flash<Redirect>, Template> {
    if Task::delete_with_id(id) {
        Ok(Flash::success(Redirect::to("/"), "Todo was deleted."))
    } else {
        Err(Template::render("index", &Context::err("Couldn't delete task.")))
    }
}

#[get("/")]
fn index(msg: Option<Flash<()>>) -> Template {
    Template::render("index", &match msg {
        Some(ref msg) => Context::raw(Some((msg.name(), msg.msg()))),
        None => Context::raw(None),
    })
}

fn main() {
    rocket::ignite()
        .mount("/", routes![index, static_files::all])
        .mount("/todo/", routes![new, toggle, delete])
        .launch();
}
