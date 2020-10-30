#[macro_use] extern crate rocket;

#[cfg(test)] mod tests;

#[get("/hello/<name>/<age>")]
fn hello(name: String, age: i8) -> String {
    format!("Hello, {} year old named {}!", age, name)
}

#[get("/hello/<name>/<age>", rank = 2)]
fn hi(name: String, age: &str) -> String {
    format!("Hi {}! Your age ({}) is kind of funky.", name, age)
}

#[launch]
fn rocket() -> rocket::Rocket {
    rocket::ignite().mount("/", routes![hi, hello])
}
