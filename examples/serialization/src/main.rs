#[macro_use] extern crate rocket;

#[cfg(test)] mod tests;

mod json;
mod msgpack;

#[launch]
fn rocket() -> _ {
    rocket::ignite()
        .attach(json::stage())
        .attach(msgpack::stage())
}
