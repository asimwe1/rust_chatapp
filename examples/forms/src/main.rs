#![feature(plugin)]
#![plugin(rocket_macros)]

extern crate rocket;

mod files;

use rocket::Rocket;
use rocket::response::Redirect;
use rocket::Error;
use rocket::form::{FromForm, FromFormValue, form_items};

#[route(GET, path = "/user/<username>")]
fn user_page(username: &str) -> String {
    format!("This is {}'s page.", username)
}

// #[derive(FromForm)] // FIXME: Make that happen.
struct UserLogin<'a> {
    username: &'a str,
    password: &'a str,
    age: Result<isize, &'a str>,
}

// will help for validation. IE, can have a type Range(1, 10) that returns an
// enum with one of: TooLow(isize), TooHigh(isize), etc.
impl<'f> FromForm<'f> for UserLogin<'f> {
    fn from_form_string(s: &'f str) -> Result<Self, Error> {
        let mut items = [("", ""); 3];
        let form_count = form_items(s, &mut items);
        if form_count != items.len() {
            return Err(Error::BadParse);
        }

        let mut username: Option<&'f str> = None;
        let mut password: Option<&'f str> = None;
        let mut age: Option<Result<isize, &'f str>> = None;
        for &(key, value) in &items {
            match key {
                "username" => username = match FromFormValue::parse(value) {
                    Ok(v) => Some(v),
                    Err(_) => return Err(Error::BadParse)
                },
                "password" => password = match FromFormValue::parse(value) {
                    Ok(v) => Some(v),
                    Err(_) => return Err(Error::BadParse)
                },
                "age" => age = match FromFormValue::parse(value) {
                    Ok(v) => Some(v),
                    Err(_) => return Err(Error::BadParse)
                },
                _ => return Err(Error::BadParse)
            }
        }

        if username.is_none() || password.is_none() {
            return Err(Error::BadParse);
        }

        Ok(UserLogin {
            username: username.unwrap(),
            password: password.unwrap(),
            age: age.unwrap(),
        })
    }
}

// TODO: Actually look at form parameters.
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
