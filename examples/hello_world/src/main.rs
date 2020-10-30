#[macro_use] extern crate rocket;

#[cfg(test)] mod tests;

#[get("/?<lang>")]
fn hello(lang: Option<&str>) -> &'static str {
    match lang {
        Some("en") | None => world(),
        Some("русский") => mir(),
        _ => "Hello, voyager!"
    }
}

#[get("/world")]
fn world() -> &'static str {
    "Hello, world!"
}

#[get("/мир")]
fn mir() -> &'static str {
    "Привет, мир!"
}

#[launch]
fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .mount("/", routes![hello])
        .mount("/hello", routes![world, mir])
}
