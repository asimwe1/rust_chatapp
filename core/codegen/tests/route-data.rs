#[macro_use] extern crate rocket;

use rocket::{Request, Data};
use rocket::local::blocking::Client;
use rocket::request::Form;
use rocket::data::{self, FromData};
use rocket::http::{RawStr, ContentType, Status};
use rocket::tokio::io::AsyncReadExt;

// Test that the data parameters works as expected.

#[derive(FromForm)]
struct Inner<'r> {
    field: &'r RawStr
}

struct Simple(String);

#[async_trait]
impl FromData for Simple {
    type Error = ();

    async fn from_data(_: &Request<'_>, data: Data) -> data::Outcome<Self, ()> {
        let mut string = String::new();
        let mut stream = data.open().take(64);
        if let Err(_) = stream.read_to_string(&mut string).await {
            return data::Outcome::Failure((Status::InternalServerError, ()));
        }

        data::Outcome::Success(Simple(string))
    }
}

#[post("/f", data = "<form>")]
fn form(form: Form<Inner<'_>>) -> String { form.field.url_decode_lossy() }

#[post("/s", data = "<simple>")]
fn simple(simple: Simple) -> String { simple.0 }

#[test]
fn test_data() {
    let rocket = rocket::ignite().mount("/", routes![form, simple]);
    let client = Client::new(rocket).unwrap();

    let response = client.post("/f")
        .header(ContentType::Form)
        .body("field=this%20is%20here")
        .dispatch();

    assert_eq!(response.into_string().unwrap(), "this is here");

    let response = client.post("/s").body("this is here").dispatch();
    assert_eq!(response.into_string().unwrap(), "this is here");

    let response = client.post("/s").body("this%20is%20here").dispatch();
    assert_eq!(response.into_string().unwrap(), "this%20is%20here");
}
