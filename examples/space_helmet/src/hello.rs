#![feature(decl_macro)]
#![feature(plugin)]
#![plugin(rocket_codegen)]
#![feature(proc_macro_non_items)]

#[macro_use] extern crate rocket;
extern crate rocket_contrib;
use rocket::http::uri::Uri;

use rocket_contrib::space_helmet::*;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

fn rocket() -> rocket::Rocket {
    let allow_uri = Uri::parse("https://mysite.example.com").unwrap();
    let report_uri = Uri::parse("https://report.example.com").unwrap();
    let helmet = SpaceHelmet::new()
        //illustrates how to disable a header by using None as the policy.
        .no_sniff(None)
        .frameguard(FramePolicy::AllowFrom(allow_uri))
        .xss_protect(XssPolicy::EnableReport(report_uri))
        //we need an hsts policy as we are using tls
        .hsts(HstsPolicy::default())
        .expect_ct(ExpectCtPolicy::default());
    rocket::ignite().mount("/", routes![index]).attach(helmet)
}

fn main() {
    rocket().launch();
}
