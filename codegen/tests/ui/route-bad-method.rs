#![feature(plugin, decl_macro)]
#![plugin(rocket_codegen)]

extern crate rocket;

#[route(FIX, "/hello")]
fn get1() -> &'static str { "hi" }

#[route("hi", "/hello")]
fn get2() -> &'static str { "hi" }

#[route("GET", "/hello")]
fn get3() -> &'static str { "hi" }

#[route(120, "/hello")]
fn get4() -> &'static str { "hi" }

#[route(CONNECT, "/hello")]
fn get5() -> &'static str { "hi" }
