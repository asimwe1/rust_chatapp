#![feature(plugin)]
#![plugin(rocket_macros)]

extern crate rocket;

mod files;

use rocket::Rocket;
use rocket::response::Redirect;
use rocket::error::Error;

#[route(GET, path = "/user/<username>")]
fn user_page(username: &str) -> String {
    format!("This is {}'s page.", username)
}

// #[derive(FormItem)] // FIXME: Make that happen.
struct UserLogin<'a> {
    username: &'a str,
    password: &'a str
}

trait FormItem: Sized {
    fn from_form_string(s: &str) -> Result<Self, Error>;
}

impl<'a> FormItem for UserLogin<'a> {
    fn from_form_string(s: &str) -> Result<Self, Error> {
        Ok(UserLogin {
            username: "this",
            password: "that"
        })
    }
}

// TODO: Actually look at form parameters.
// FIXME: fn login<'a>(user: UserLogin<'a>)
#[route(POST, path = "/login", form = "<user>")]
fn login(user: UserLogin) -> Result<Redirect, String> {
    match user.username {
        "Sergio" => match user.password {
            "password" => Ok(Redirect::other("/user/some_name")),
            _ => Err("Wrong password!".to_string())
        },
        _ => Err(format!("Unrecognized user, '{}'.", user.username))
    }
}

fn main() {
    let mut rocket = Rocket::new("localhost", 8000);
    rocket.mount("/", routes![files::index, files::files, user_page, login]);
    rocket.launch();
}
