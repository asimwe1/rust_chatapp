#![feature(proc_macro_diagnostic)]
#![feature(crate_visibility_modifier)]
#![recursion_limit="128"]

#[macro_use] extern crate quote;
#[macro_use] extern crate derive_utils;
extern crate proc_macro;
extern crate rocket_http;

mod derive;
mod http_codegen;

crate use derive_utils::{syn, proc_macro2};

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
