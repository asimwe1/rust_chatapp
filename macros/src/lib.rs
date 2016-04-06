#![crate_type = "dylib"]
#![feature(quote, concat_idents, plugin_registrar, rustc_private)]

#[macro_use] extern crate syntax;
extern crate syntax_ext;
extern crate rustc;
extern crate rustc_plugin;
extern crate rocket;

#[macro_use] mod utils;
mod routes_macro;
mod errors_macro;
mod route_decorator;
mod error_decorator;
mod derive_form;

use rustc_plugin::Registry;
use syntax::ext::base::SyntaxExtension;
use syntax::parse::token::intern;

use routes_macro::routes_macro;
use errors_macro::errors_macro;
use route_decorator::route_decorator;
use error_decorator::error_decorator;
use derive_form::from_form_derive;

const ROUTE_STRUCT_PREFIX: &'static str = "static_rocket_route_info_for_";
const CATCH_STRUCT_PREFIX: &'static str = "static_rocket_catch_info_for_";
const ROUTE_FN_PREFIX: &'static str = "rocket_route_fn_";
const CATCH_FN_PREFIX: &'static str = "rocket_catch_fn_";

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_syntax_extension(intern("route"),
        SyntaxExtension::MultiDecorator(Box::new(route_decorator)));
    reg.register_syntax_extension(intern("error"),
        SyntaxExtension::MultiDecorator(Box::new(error_decorator)));
    reg.register_syntax_extension(intern("derive_FromForm"),
        SyntaxExtension::MultiDecorator(Box::new(from_form_derive)));
    reg.register_macro("routes", routes_macro);
    reg.register_macro("errors", errors_macro);
}
