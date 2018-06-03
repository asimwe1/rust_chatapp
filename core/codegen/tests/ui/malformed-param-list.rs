#![feature(plugin, decl_macro)]
#![plugin(rocket_codegen)]

#[get("/><")]
fn get() -> &'static str { "hi" }

#[get("/<id><")]
fn get1(id: usize) -> &'static str { "hi" }

#[get("/<<<<id><")]
fn get2(id: usize) -> &'static str { "hi" }

#[get("/<!>")]
fn get3() -> &'static str { "hi" }

#[get("/<_>")]
fn get4() -> &'static str { "hi" }

#[get("/<1>")]
fn get5() -> &'static str { "hi" }

#[get("/<>name><")]
fn get6() -> &'static str { "hi" }

#[get("/<name>:<id>")]
fn get7() -> &'static str { "hi" }

#[get("/<>")]
fn get8() -> &'static str { "hi" }
