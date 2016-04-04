#![crate_type = "dylib"]
#![feature(quote, concat_idents, plugin_registrar, rustc_private)]

#[macro_use] extern crate syntax;
extern crate syntax_ext;
extern crate rustc;
extern crate rustc_plugin;
extern crate rocket;

#[macro_use] mod utils;
mod routes_macro;
mod route_decorator;
mod derive_form;

use rustc_plugin::Registry;
use syntax::ext::base::SyntaxExtension;
use syntax::parse::token::intern;

use routes_macro::routes_macro;
use route_decorator::route_decorator;
use derive_form::from_form_derive;

const STRUCT_PREFIX: &'static str = "static_rocket_route_info_for_";
const FN_PREFIX: &'static str = "rocket_route_fn_";

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_syntax_extension(intern("route"),
        SyntaxExtension::MultiDecorator(Box::new(route_decorator)));
    reg.register_syntax_extension(intern("derive_FromForm"),
        SyntaxExtension::MultiDecorator(Box::new(from_form_derive)));
    reg.register_macro("routes", routes_macro);
}
