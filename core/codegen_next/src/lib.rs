#![feature(proc_macro_diagnostic, proc_macro_span)]
#![feature(crate_visibility_modifier)]
#![feature(transpose_result)]
#![feature(rustc_private)]
#![recursion_limit="128"]

#[macro_use] extern crate quote;
#[macro_use] extern crate derive_utils;
extern crate indexmap;
extern crate proc_macro;
extern crate rocket_http as http;
extern crate indexmap;

extern crate syntax_pos;

#[macro_use] mod proc_macro_ext;
mod derive;
mod attribute;
mod bang;
mod http_codegen;
mod syn_ext;

use http::Method;
use proc_macro::TokenStream;
crate use derive_utils::proc_macro2;

crate static ROUTE_STRUCT_PREFIX: &str = "static_rocket_route_info_for_";
crate static CATCH_STRUCT_PREFIX: &str = "static_rocket_catch_info_for_";
crate static CATCH_FN_PREFIX: &str = "rocket_catch_fn_";
crate static ROUTE_FN_PREFIX: &str = "rocket_route_fn_";
crate static URI_MACRO_PREFIX: &str = "rocket_uri_macro_";
crate static ROCKET_PARAM_PREFIX: &str = "__rocket_param_";

macro_rules! route_attribute {
    ($name:ident => $method:expr) => (
        #[proc_macro_attribute]
        pub fn $name(args: TokenStream, input: TokenStream) -> TokenStream {
            attribute::route::route_attribute($method, args, input)
        }
    )
}
route_attribute!(route => None);
route_attribute!(get => Method::Get);
route_attribute!(put => Method::Put);
route_attribute!(post => Method::Post);
route_attribute!(delete => Method::Delete);
route_attribute!(head => Method::Head);
route_attribute!(patch => Method::Patch);
route_attribute!(options => Method::Options);

#[proc_macro_derive(FromFormValue, attributes(form))]
pub fn derive_from_form_value(input: TokenStream) -> TokenStream {
    derive::from_form_value::derive_from_form_value(input)
}

#[proc_macro_derive(FromForm, attributes(form))]
pub fn derive_from_form(input: TokenStream) -> TokenStream {
    derive::from_form::derive_from_form(input)
}

#[proc_macro_derive(Responder, attributes(response))]
pub fn derive_responder(input: TokenStream) -> TokenStream {
    derive::responder::derive_responder(input)
}

#[proc_macro_attribute]
pub fn catch(args: TokenStream, input: TokenStream) -> TokenStream {
    attribute::catch::catch_attribute(args, input)
}

#[proc_macro]
pub fn routes(input: TokenStream) -> TokenStream {
    bang::routes_macro(input)
}

#[proc_macro]
pub fn catchers(input: TokenStream) -> TokenStream {
    bang::catchers_macro(input)
}

#[proc_macro]
pub fn uri(input: TokenStream) -> TokenStream {
    bang::uri_macro(input)
}

#[proc_macro]
pub fn rocket_internal_uri(input: TokenStream) -> TokenStream {
    bang::uri_internal_macro(input)
}
