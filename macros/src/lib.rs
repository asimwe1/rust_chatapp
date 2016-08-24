#![crate_type = "dylib"]
#![feature(quote, concat_idents, plugin_registrar, rustc_private)]

#[macro_use] extern crate syntax;
#[macro_use] extern crate log;
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
mod meta_item_parser;

use rustc_plugin::Registry;
use syntax::ext::base::SyntaxExtension;
use syntax::parse::token::intern;

use routes_macro::routes_macro;
use errors_macro::errors_macro;
use route_decorator::*;
use error_decorator::error_decorator;
use derive_form::from_form_derive;

const ROUTE_STRUCT_PREFIX: &'static str = "static_rocket_route_info_for_";
const CATCH_STRUCT_PREFIX: &'static str = "static_rocket_catch_info_for_";
const ROUTE_FN_PREFIX: &'static str = "rocket_route_fn_";
const CATCH_FN_PREFIX: &'static str = "rocket_catch_fn_";

macro_rules! register_decorators {
    ($registry:expr, $($name:expr => $func:expr),+) => (
        $($registry.register_syntax_extension(intern($name),
                SyntaxExtension::MultiDecorator(Box::new($func)));
         )+
    )
}

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_macro("routes", routes_macro);
    reg.register_macro("errors", errors_macro);

    register_decorators!(reg,
        "derive_FromForm" => from_form_derive,
        "route" => generic_route_decorator,
        "error" => error_decorator,

        "GET" => get_decorator,
        "PUT" => put_decorator,
        "POST" => post_decorator,
        "DELETE" => delete_decorator,
        "OPTIONS" => options_decorator,
        "HEAD" => head_decorator,
        "TRACE" => trace_decorator,
        "CONNECT" => connect_decorator,
        "PATCH" => patch_decorator
    );
}
