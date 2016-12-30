#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;

#[get("/")]
fn get() {  }

#[put("/")]
fn put() {  }

#[post("/")]
fn post() {  }

#[delete("/")]
fn delete() {  }

#[head("/")]
fn head() {  }

#[patch("/")]
fn patch() {  }

// TODO: Allow this once Diesel incompatibility is fixed.
// #[options("/")]
// fn options() {  }

fn main() { }
