use std::collections::HashMap;

use rocket::Request;
use rocket::response::Redirect;
use rocket_contrib::templates::{Template, tera::Tera};

#[derive(serde::Serialize)]
struct TemplateContext<'r> {
    title: &'r str,
    name: &'r str,
    items: Vec<&'r str>
}

#[get("/")]
pub fn index() -> Redirect {
    Redirect::to(uri!("/tera", hello: name = "Your Name"))
}

#[get("/hello/<name>")]
pub fn hello(name: &str) -> Template {
    Template::render("tera/index", &TemplateContext {
        name,
        title: "Hello",
        items: vec!["One", "Two", "Three"],
    })
}

#[catch(404)]
pub fn not_found(req: &Request<'_>) -> Template {
    let mut map = HashMap::new();
    map.insert("path", req.uri().path());
    Template::render("tera/error/404", &map)
}

pub fn customize(_tera: &mut Tera) {
    /* register helpers, and so on */
}
