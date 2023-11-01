use rocket::form::Form;
use rocket::response::Redirect;
use rocket::http::CookieJar;
use rocket_dyn_templates::{Template, context};

#[macro_export]
macro_rules! message_uri {
    ($($t:tt)*) => (rocket::uri!("/message", $crate::message:: $($t)*))
}

pub use message_uri as uri;

#[post("/", data = "<message>")]
fn submit(cookies: &CookieJar<'_>, message: Form<&str>) -> Redirect {
    cookies.add(("message", message.to_string()));
    Redirect::to(uri!(index))
}

#[get("/")]
fn index(cookies: &CookieJar<'_>) -> Template {
    let message = cookies.get("message").map(|c| c.value());
    Template::render("message", context! { message })
}

pub fn routes() -> Vec<rocket::Route> {
    routes![submit, index]
}
