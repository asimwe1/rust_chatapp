#[macro_use] extern crate rocket;

#[cfg(test)] mod tests;

mod json;
mod msgpack;

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(json::stage())
        .attach(msgpack::stage())
}
