#![feature(plugin)]
#![plugin(rocket_codegen)]

#[get("/><")] //~ ERROR malformed
fn get() -> &'static str { "hi" }

#[get("/<name><")] //~ ERROR malformed
fn get1(name: &str) -> &'static str { "hi" }

#[get("/<<<<name><")] //~ ERROR identifiers
fn get2(name: &str) -> &'static str { "hi" }

#[get("/<!>")] //~ ERROR identifiers
fn get3() -> &'static str { "hi" }

#[get("/<_>")] //~ ERROR ignored
fn get4() -> &'static str { "hi" }

#[get("/<1>")] //~ ERROR identifiers
fn get5() -> &'static str { "hi" }

#[get("/<>name><")] //~ ERROR cannot be empty
fn get6() -> &'static str { "hi" }

fn main() {  }
