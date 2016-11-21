#![crate_type = "dylib"]
#![feature(quote, concat_idents, plugin_registrar, rustc_private, unicode)]
#![feature(custom_attribute)]
#![allow(unused_attributes)]

#[macro_use] extern crate syntax;
#[macro_use] extern crate log;
extern crate syntax_ext;
extern crate rustc;
extern crate rustc_plugin;
extern crate rocket;

#[macro_use] mod utils;
mod parser;
mod macros;
mod decorators;

use rustc_plugin::Registry;
use syntax::ext::base::SyntaxExtension;
use syntax::parse::token::intern;

const PARAM_PREFIX: &'static str = "rocket_param_";
const ROUTE_STRUCT_PREFIX: &'static str = "static_rocket_route_info_for_";
const CATCH_STRUCT_PREFIX: &'static str = "static_rocket_catch_info_for_";
const ROUTE_FN_PREFIX: &'static str = "rocket_route_fn_";
const CATCH_FN_PREFIX: &'static str = "rocket_catch_fn_";

macro_rules! register_decorators {
    ($registry:expr, $($name:expr => $func:ident),+) => (
        $($registry.register_syntax_extension(intern($name),
                SyntaxExtension::MultiDecorator(Box::new(decorators::$func)));
         )+
    )
}

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_macro("routes", macros::routes);
    reg.register_macro("errors", macros::errors);

    register_decorators!(reg,
        "derive_FromForm" => from_form_derive,

        "error" => error_decorator,
        "route" => route_decorator,
        "get" => get_decorator,
        "put" => put_decorator,
        "post" => post_decorator,
        "delete" => delete_decorator,
        "patch" => patch_decorator
    );
}
