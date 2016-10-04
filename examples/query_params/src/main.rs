#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]

extern crate rocket;

#[derive(FromForm)]
struct Person<'r> {
    name: &'r str,
    age: Option<u8>
}

#[get("/hello?<person>")]
fn hello(person: Person) -> String {
    if let Some(age) = person.age {
        format!("Hello, {} year old named {}!", age, person.name)
    } else {
        format!("Hello {}!", person.name)
    }
}

fn main() {
    rocket::ignite().mount("/", routes![hello]).launch()
}
