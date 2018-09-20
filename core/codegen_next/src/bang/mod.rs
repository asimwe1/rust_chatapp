use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;

use derive_utils::{syn, Spanned, Result};
use self::syn::{Path, punctuated::Punctuated, parse::Parser, token::Comma};
use syn_ext::{IdentExt, syn_to_diag};

mod uri;
mod uri_parsing;

fn _prefixed_vec(prefix: &str, input: TokenStream, ty: &TokenStream2) -> Result<TokenStream2> {
    // Parse a comma-separated list of paths.
    let mut paths = <Punctuated<Path, Comma>>::parse_terminated
        .parse(input)
        .map_err(syn_to_diag)?;

    // Prefix the last segment in each path with `prefix`.
    for path in paths.iter_mut() {
        let mut last_seg = path.segments.last_mut().expect("last path segment");
        last_seg.value_mut().ident = last_seg.value().ident.prepend(prefix);
    }

    // Return a `vec!` of the prefixed, mapped paths.
    let prefixed_mapped_paths = paths.iter()
        .map(|path| quote_spanned!(path.span().into() => #ty::from(&#path)));

    Ok(quote!(vec![#(#prefixed_mapped_paths),*]))
}

fn prefixed_vec(prefix: &str, input: TokenStream, ty: TokenStream2) -> TokenStream {
    let vec = _prefixed_vec(prefix, input, &ty)
        .map_err(|diag| diag.emit())
        .unwrap_or_else(|_| quote!(vec![]));

    quote!({
        let __vector: Vec<#ty> = #vec;
        __vector
    }).into()
}

pub static ROUTE_STRUCT_PREFIX: &str = "static_rocket_route_info_for_";

pub fn routes_macro(input: TokenStream) -> TokenStream {
    prefixed_vec(ROUTE_STRUCT_PREFIX, input, quote!(::rocket::Route))
}

pub static CATCH_STRUCT_PREFIX: &str = "static_rocket_catch_info_for_";

pub fn catchers_macro(input: TokenStream) -> TokenStream {
    prefixed_vec(CATCH_STRUCT_PREFIX, input, quote!(::rocket::Catcher))
}

pub fn uri_macro(input: TokenStream) -> TokenStream {
    uri::_uri_macro(input)
        .map_err(|diag| diag.emit())
        .unwrap_or_else(|_| quote!(()).into())
}

pub fn uri_internal_macro(input: TokenStream) -> TokenStream {
    uri::_uri_internal_macro(input)
        .map_err(|diag| diag.emit())
        .unwrap_or_else(|_| quote!(()).into())
}
