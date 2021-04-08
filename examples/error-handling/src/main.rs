#[macro_use] extern crate rocket;

#[cfg(test)] mod tests;

use rocket::Request;
use rocket::response::{content, status};
use rocket::http::Status;

#[get("/hello/<name>/<age>")]
fn hello(name: &str, age: i8) -> String {
    format!("Hello, {} year old named {}!", age, name)
}

#[get("/<code>")]
fn forced_error(code: u16) -> Status {
    Status::raw(code)
}

#[catch(404)]
fn general_not_found() -> content::Html<&'static str> {
    content::Html(r#"
        <p>Hmm... What are you looking for?</p>
        Say <a href="/hello/Sergio/100">hello!</a>
    "#)
}

#[catch(404)]
fn hello_not_found(req: &Request<'_>) -> content::Html<String> {
    content::Html(format!("\
        <p>Sorry, but '{}' is not a valid path!</p>\
        <p>Try visiting /hello/&lt;name&gt;/&lt;age&gt; instead.</p>",
        req.uri()))
}

#[catch(default)]
fn sergio_error() -> &'static str {
    "I...don't know what to say."
}

#[catch(default)]
fn default_catcher(status: Status, req: &Request<'_>) -> status::Custom<String> {
    let msg = format!("{} - {} ({})", status.code, status.reason, req.uri());
    status::Custom(status, msg)
}

fn rocket() -> rocket::Rocket {
    rocket::build()
        // .mount("/", routes![hello, hello]) // uncoment this to get an error
        .mount("/", routes![hello, forced_error])
        .register("/", catchers![general_not_found, default_catcher])
        .register("/hello", catchers![hello_not_found])
        .register("/hello/Sergio", catchers![sergio_error])
}

#[rocket::main]
async fn main() {
    if let Err(e) = rocket().launch().await {
        println!("Whoops! Rocket didn't launch!");
        println!("Error: {:?}", e);
    };
}
