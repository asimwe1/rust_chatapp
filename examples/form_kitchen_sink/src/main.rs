#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]

extern crate rocket;

use rocket::request::{Form, FromFormValue};
use rocket::response::NamedFile;
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
struct FormInput {
    checkbox: bool,
    number: usize,
    radio: FormOption,
    password: String,
    textarea: String,
    select: FormOption,
}

#[post("/", data = "<sink>")]
fn sink(sink: Result<Form<FormInput>, Option<String>>) -> String {
    match sink {
        Ok(form) => format!("{:?}", form.get()),
        Err(Some(f)) => format!("Invalid form input: {}", f),
        Err(None) => format!("Form input was invalid UTF8."),
    }
}

#[get("/")]
fn index() -> io::Result<NamedFile> {
    NamedFile::open("static/index.html")
}

fn main() {
    rocket::ignite()
        .mount("/", routes![index, sink])
        .launch();
}
