#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]

extern crate rocket;

use rocket::{Rocket, Request};
use rocket::response::NamedFile;
use rocket::form::FromFormValue;
use std::io;

// TODO: Make deriving `FromForm` for this enum possible.
#[derive(Debug)]
enum FormOption {
    A, B, C
}

impl<'v> FromFormValue<'v> for FormOption {
    type Error = &'v str;

    fn from_form_value(v: &'v str) -> Result<Self, Self::Error> {
        let variant = match v {
            "a" => FormOption::A,
            "b" => FormOption::B,
            "c" => FormOption::C,
            _ => return Err(v)
        };

        Ok(variant)
    }
}

#[derive(Debug, FromForm)]
struct FormInput<'r> {
    checkbox: bool,
    number: usize,
    radio: FormOption,
    password: &'r str,
    textarea: String,
    select: FormOption,
}

#[post("/", form = "<sink>")]
fn sink(sink: FormInput) -> String {
    format!("{:?}", sink)
}

#[post("/", rank = 2)]
fn sink2(request: &Request) -> &'static str {
    println!("form: {:?}", std::str::from_utf8(request.data.as_slice()));
    "Sorry, the form is invalid."
}

#[get("/")]
fn index() -> io::Result<NamedFile> {
    NamedFile::open("static/index.html")
}

fn main() {
    let mut rocket = Rocket::new("localhost", 8000);
    rocket.mount("/", routes![index, sink, sink2]);
    rocket.launch();
}
