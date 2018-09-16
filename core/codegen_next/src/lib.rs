#![feature(proc_macro_diagnostic)]
#![feature(crate_visibility_modifier)]
#![recursion_limit="128"]

#[macro_use] extern crate quote;
#[macro_use] extern crate derive_utils;
extern crate proc_macro;
extern crate rocket_http;

mod derive;
mod attribute;
mod bang;
mod http_codegen;
mod syn_ext;

crate use derive_utils::proc_macro2;

use proc_macro::TokenStream;

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
