#[macro_use] extern crate rocket;
#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_migrations;
#[macro_use] extern crate rocket_contrib;

#[cfg(test)]
mod tests;
mod task;

use rocket::Rocket;
use rocket::fairing::AdHoc;
use rocket::request::FlashMessage;
use rocket::response::{Flash, Redirect};
use rocket::form::Form;

use rocket_contrib::templates::Template;
use rocket_contrib::serve::{StaticFiles, crate_relative};

use crate::task::{Task, Todo};

#[database("sqlite_database")]
pub struct DbConn(diesel::SqliteConnection);

#[derive(Debug, serde::Serialize)]
struct Context {
    msg: Option<(String, String)>,
    tasks: Vec<Task>
}

impl Context {
    pub async fn err<M: std::fmt::Display>(conn: &DbConn, msg: M) -> Context {
        Context {
            msg: Some(("error".into(), msg.to_string())),
            tasks: Task::all(conn).await.unwrap_or_default()
        }
    }

    pub async fn raw(conn: &DbConn, msg: Option<(String, String)>) -> Context {
        match Task::all(conn).await {
            Ok(tasks) => Context { msg, tasks },
            Err(e) => {
                error_!("DB Task::all() error: {}", e);
                Context {
                    msg: Some(("error".into(), "Fail to access database.".into())),
                    tasks: vec![]
                }
            }
        }
    }
}

#[post("/", data = "<todo_form>")]
async fn new(todo_form: Form<Todo>, conn: DbConn) -> Flash<Redirect> {
    let todo = todo_form.into_inner();
    if todo.description.is_empty() {
        Flash::error(Redirect::to("/"), "Description cannot be empty.")
    } else if let Err(e) = Task::insert(todo, &conn).await {
        error_!("DB insertion error: {}", e);
        Flash::error(Redirect::to("/"), "Todo could not be inserted due an internal error.")
    } else {
        Flash::success(Redirect::to("/"), "Todo successfully added.")
    }
}

#[put("/<id>")]
async fn toggle(id: i32, conn: DbConn) -> Result<Redirect, Template> {
    match Task::toggle_with_id(id, &conn).await {
        Ok(_) => Ok(Redirect::to("/")),
        Err(e) => {
            error_!("DB toggle({}) error: {}", id, e);
            Err(Template::render("index", Context::err(&conn, "Failed to toggle task.").await))
        }
    }
}

#[delete("/<id>")]
async fn delete(id: i32, conn: DbConn) -> Result<Flash<Redirect>, Template> {
    match Task::delete_with_id(id, &conn).await {
        Ok(_) => Ok(Flash::success(Redirect::to("/"), "Todo was deleted.")),
        Err(e) => {
            error_!("DB deletion({}) error: {}", id, e);
            Err(Template::render("index", Context::err(&conn, "Failed to delete task.").await))
        }
    }
}

#[get("/")]
async fn index(msg: Option<FlashMessage<'_>>, conn: DbConn) -> Template {
    let msg = msg.map(|m| (m.name().to_string(), m.msg().to_string()));
    Template::render("index", Context::raw(&conn, msg).await)
}

async fn run_db_migrations(rocket: Rocket) -> Result<Rocket, Rocket> {
    // This macro from `diesel_migrations` defines an `embedded_migrations`
    // module containing a function named `run`. This allows the example to be
    // run and tested without any outside setup of the database.
    embed_migrations!();

    let conn = DbConn::get_one(&rocket).await.expect("database connection");
    match conn.run(|c| embedded_migrations::run(c)).await {
        Ok(()) => Ok(rocket),
        Err(e) => {
            error!("Failed to run database migrations: {:?}", e);
            Err(rocket)
        }
    }
}

#[launch]
fn rocket() -> Rocket {
    rocket::ignite()
        .attach(DbConn::fairing())
        .attach(Template::fairing())
        .attach(AdHoc::on_launch("Database Migrations", run_db_migrations))
        .mount("/", StaticFiles::from(crate_relative!("/static")))
        .mount("/", routes![index])
        .mount("/todo", routes![new, toggle, delete])
}
