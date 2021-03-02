#![warn(rust_2018_idioms)]
#[cfg(test)] mod tests;

#[rocket::get("/")]
fn hello() -> &'static str {
    "Hello, Rust 2018!"
}

#[rocket::launch]
fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .mount("/", rocket::routes![hello])
        .register(rocket::catchers![not_found])
}

#[rocket::catch(404)]
fn not_found(_req: &'_ rocket::Request<'_>) -> &'static str {
    "404 Not Found"
}
