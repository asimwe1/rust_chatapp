#![feature(proc_macro_non_items, proc_macro_gen, decl_macro)]

#[macro_use] extern crate rocket;

mod common;

#[test]
fn test_staging_config() {
    common::test_config(rocket::config::Environment::Staging);
}
