#![feature(plugin, decl_macro)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate rocket_contrib;

#[cfg(test)] mod tests;

use rocket_contrib::static_files::StaticFiles;

fn rocket() -> rocket::Rocket {
    rocket::ignite().mount("/", StaticFiles::from("static"))
}

fn main() {
    rocket().launch();
}
