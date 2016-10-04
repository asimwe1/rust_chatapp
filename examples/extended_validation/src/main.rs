#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]

extern crate rocket;

mod files;

use rocket::response::Redirect;
use rocket::form::FromFormValue;

#[derive(Debug)]
struct StrongPassword<'r>(&'r str);

#[derive(Debug)]
struct AdultAge(isize);

#[derive(FromForm)]
struct UserLogin<'r> {
    username: &'r str,
    password: Result<StrongPassword<'r>, &'static str>,
    age: Result<AdultAge, &'static str>,
}

impl<'v> FromFormValue<'v> for StrongPassword<'v> {
    type Error = &'static str;

    fn from_form_value(v: &'v str) -> Result<Self, Self::Error> {
        if v.len() < 8 {
            Err("Too short!")
        } else {
            Ok(StrongPassword(v))
        }
    }
}

impl<'v> FromFormValue<'v> for AdultAge {
    type Error = &'static str;

    fn from_form_value(v: &'v str) -> Result<Self, Self::Error> {
        let age = match isize::from_form_value(v) {
            Ok(v) => v,
            Err(_) => return Err("Age value is not a number."),
        };

        match age > 20 {
            true => Ok(AdultAge(age)),
            false => Err("Must be at least 21."),
        }
    }
}

#[post("/login", form = "<user>")]
fn login(user: UserLogin) -> Result<Redirect, String> {
    if user.age.is_err() {
        return Err(String::from(user.age.unwrap_err()));
    }

    if user.password.is_err() {
        return Err(String::from(user.password.unwrap_err()));
    }

    match user.username {
        "Sergio" => {
            match user.password.unwrap().0 {
                "password" => Ok(Redirect::other("/user/Sergio")),
                _ => Err("Wrong password!".to_string()),
            }
        }
        _ => Err(format!("Unrecognized user, '{}'.", user.username)),
    }
}

#[get("/user/<username>")]
fn user_page(username: &str) -> String {
    format!("This is {}'s page.", username)
}

fn main() {
    let mut rocket = rocket::ignite();
    rocket.mount("/", routes![files::index, files::files, user_page, login]);
    rocket.launch();
}
