#![feature(plugin, custom_derive)]
#![plugin(rocket_macros)]

extern crate rocket;

use rocket::Rocket;
use rocket::response::Redirect;

#[derive(FromForm)]
struct Todo<'r> {
    description: &'r str,
}

#[route(POST, path = "/todo", form = "<todo>")]
fn new_todo(todo: Todo) -> Result<Redirect, &'static str> {
    // if todos.add(todo).is_ok() {
    //     Ok(Redirect::to("/"))
    // } else {
    //     Err("Could not add todo to list.")
    // }

    Ok(Redirect::to("/"))
}

#[route(GET, path = "/todos")]
fn list_todos() -> &'static str {
    "List all of the todos here!"
}

#[route(GET, path = "/")]
fn index() -> Redirect {
    Redirect::to("/todos")
}

fn main() {
    let mut rocket = Rocket::new("localhost", 8000);
    rocket.mount("/", routes![index, list_todos, new_todo]);
    rocket.launch();
}
