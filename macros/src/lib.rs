#![crate_type = "dylib"]
#![feature(quote, concat_idents, plugin_registrar, rustc_private)]

#[macro_use] extern crate syntax;
extern crate rustc;
extern crate rustc_plugin;
extern crate rocket;

#[macro_use] mod utils;
mod routes_macro;
mod route_decorator;

use rustc_plugin::Registry;
use syntax::ext::base::SyntaxExtension;
use syntax::parse::token::intern;

use routes_macro::routes_macro;
use route_decorator::route_decorator;

const STRUCT_PREFIX: &'static str = "static_rocket_route_info_for_";
const FN_PREFIX: &'static str = "rocket_route_fn_";

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_syntax_extension(intern("route"),
        SyntaxExtension::MultiDecorator(Box::new(route_decorator)));
    reg.register_macro("routes", routes_macro);
}
