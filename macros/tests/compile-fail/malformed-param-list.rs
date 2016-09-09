#![feature(plugin)]
#![plugin(rocket_macros)]

#[get("/><")] //~ ERROR malformed
fn get() -> &'static str { "hi" }

#[get("/<name><")] //~ ERROR malformed
fn get(name: &str) -> &'static str { "hi" }

#[get("/<<<<name><")] //~ ERROR identifiers
fn get(name: &str) -> &'static str { "hi" }

#[get("/<!>")] //~ ERROR identifiers
fn get() -> &'static str { "hi" }

#[get("/<_>")] //~ ERROR ignored
fn get() -> &'static str { "hi" }

#[get("/<1>")] //~ ERROR identifiers
fn get() -> &'static str { "hi" }

#[get("/<>name><")] //~ ERROR cannot be empty
fn get() -> &'static str { "hi" }

fn main() {  }
