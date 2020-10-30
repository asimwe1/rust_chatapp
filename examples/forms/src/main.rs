#[macro_use]extern crate rocket;

use rocket::http::Status;
use rocket::form::{Form, Contextual, FromForm, FromFormField, Context};
use rocket::data::TempFile;

use rocket_contrib::serve::{StaticFiles, crate_relative};
use rocket_contrib::templates::Template;

#[derive(Debug, FromForm)]
struct Password<'v> {
    #[field(validate = len(6..))]
    #[field(validate = eq(self.second))]
    first: &'v str,
    #[field(validate = eq(self.first))]
    second: &'v str,
}

#[derive(Debug, FromFormField)]
enum Rights {
    Public,
    Reserved,
    Exclusive,
}

#[derive(Debug, FromFormField)]
enum Category {
    Biology,
    Chemistry,
    Physics,
    #[field(value = "CS")]
    ComputerScience,
}

#[derive(Debug, FromForm)]
struct Submission<'v> {
    #[field(validate = len(1..))]
    title: &'v str,
    date: time::Date,
    #[field(validate = len(1..=250))]
    r#abstract: &'v str,
    #[field(validate = ext("pdf"))]
    file: TempFile<'v>,
    #[field(validate = len(1..))]
    category: Vec<Category>,
    rights: Rights,
    ready: bool,
}

#[derive(Debug, FromForm)]
struct Account<'v> {
    #[field(validate = len(1..))]
    name: &'v str,
    password: Password<'v>,
    #[field(validate = contains('@'))]
    #[field(validate = omits(self.password.first))]
    email: &'v str,
}

#[derive(Debug, FromForm)]
struct Submit<'v> {
    account: Account<'v>,
    submission: Submission<'v>,
}

#[get("/")]
fn index<'r>() -> Template {
    Template::render("index", &Context::default())
}

#[post("/", data = "<form>")]
fn submit<'r>(form: Form<Contextual<'r, Submit<'r>>>) -> (Status, Template) {
    let template = match form.value {
        Some(ref submission) => {
            println!("submission: {:#?}", submission);
            Template::render("success", &form.context)
        }
        None => Template::render("index", &form.context),
    };

    (form.context.status(), template)
}

#[launch]
fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .mount("/", routes![index, submit])
        .attach(Template::fairing())
        .mount("/", StaticFiles::from(crate_relative!("/static")))
}
