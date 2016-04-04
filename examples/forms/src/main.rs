#![feature(plugin, custom_derive)]
#![plugin(rocket_macros)]

extern crate rocket;

mod files;

use rocket::Rocket;
use rocket::response::Redirect;

#[route(GET, path = "/user/<username>")]
fn user_page(username: &str) -> String {
    format!("This is {}'s page.", username)
}

#[derive(FromForm)]
struct UserLogin<'r> {
    username: &'r str,
    password: &'r str,
    age: Result<isize, &'r str>,
}

// FIXME: fn login<'a>(user: UserLogin<'a>)
#[route(POST, path = "/login", form = "<user>")]
fn login(user: UserLogin) -> Result<Redirect, String> {
    if user.age.is_err() {
        let input = user.age.unwrap_err();
        return Err(format!("'{}' is not a valid age integer.", input));
    }

    if user.age.unwrap() < 20 {
        return Err(format!("Sorry, {} is too young!", user.age.unwrap()));
    }

    match user.username {
        "Sergio" => match user.password {
            "password" => Ok(Redirect::other("/user/Sergio")),
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
