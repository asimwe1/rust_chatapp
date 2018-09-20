use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use std::fmt::Display;

use derive_utils::{syn, Result};
use quote::ToTokens;
use syn_ext::{IdentExt, syn_to_diag};

use self::syn::{Expr, Ident, Type};
use self::syn::spanned::Spanned as SynSpanned;
use super::uri_parsing::*;

use rocket_http::uri::Origin;
use rocket_http::ext::IntoOwned;

const URI_INFO_MACRO_PREFIX: &str = "rocket_uri_for_";

crate fn _uri_macro(input: TokenStream) -> Result<TokenStream> {
    let args: TokenStream2 = input.clone().into();

    let params = match syn::parse::<UriParams>(input) {
        Ok(p) => p,
        Err(e) => return Err(syn_to_diag(e)),
    };
    let mut path = params.route_path;
    {
        let mut last_seg = path.segments.last_mut().expect("last path segment");
        last_seg.value_mut().ident = last_seg.value().ident.prepend(URI_INFO_MACRO_PREFIX);
    }

    // It's incredibly important we use this span as the Span for the generated
    // code so that errors from the `internal` call show up on the user's code.
    Ok(quote_spanned!(args.span().into() => {
        #path!(#args)
    }).into())
}

macro_rules! p {
    ("parameter", $num:expr) => (
        if $num == 1 { "parameter" } else { "parameters" }
    );

    ($num:expr, "was") => (
        if $num == 1 { "1 was".into() } else { format!("{} were", $num) }
    );

    ($num:expr, "parameter") => (
        if $num == 1 { "1 parameter".into() } else { format!("{} parameters", $num) }
    )
}

fn extract_exprs(internal: &InternalUriParams) -> Result<Vec<Expr>> {
    let route_name = &internal.uri_params.route_path;
    match internal.validate() {
        Validation::Ok(exprs) => Ok(exprs),
        Validation::Unnamed(expected, actual) => {
            let mut diag = internal.uri_params.args_span().error(
                format!("`{}` route uri expects {} but {} supplied",
                         route_name.clone().into_token_stream(), p!(expected, "parameter"), p!(actual, "was")));

            if expected > 0 {
                let ps = p!("parameter", expected);
                diag = diag.note(format!("expected {}: {}", ps, internal.fn_args_str()));
            }

            Err(diag)
        }
        Validation::Named(missing, extra, dup) => {
            let e = format!("invalid parameters for `{}` route uri", route_name.clone().into_token_stream());
            let mut diag = internal.uri_params.args_span().error(e);
            diag = diag.note(format!("uri parameters are: {}", internal.fn_args_str()));

            fn join<S: Display, T: Iterator<Item = S>>(iter: T) -> (&'static str, String) {
                let items: Vec<_> = iter.map(|i| format!("`{}`", i)).collect();
                (p!("parameter", items.len()), items.join(", "))
            }

            if !extra.is_empty() {
                let (ps, msg) = join(extra.iter());
                let spans: Vec<_> = extra.iter().map(|ident| ident.span().unstable()).collect();
                diag = diag.span_help(spans, format!("unknown {}: {}", ps, msg));
            }

            if !dup.is_empty() {
                let (ps, msg) = join(dup.iter());
                let spans: Vec<_> = dup.iter().map(|ident| ident.span().unstable()).collect();
                diag = diag.span_help(spans, format!("duplicate {}: {}", ps, msg));
            }

            if !missing.is_empty() {
                let (ps, msg) = join(missing.iter());
                diag = diag.help(format!("missing {}: {}", ps, msg));
            }

            Err(diag)
        }
    }
}

// Validates the mount path and the URI and returns a single Origin URI with
// both paths concatinated. Validation should always succeed since this macro
// can only be called if the route attribute succeed, which implies that the
// route URI was valid.
fn extract_origin(internal: &InternalUriParams) -> Result<Origin<'static>> {
    let base_uri = match internal.uri_params.mount_point {
        Some(ref base) => Origin::parse(&base.value())
            .map_err(|_| base.span().unstable().error("invalid path URI"))?
            .into_owned(),
        None => Origin::dummy()
    };

    Origin::parse_route(&format!("{}/{}", base_uri, internal.uri))
        .map(|o| o.to_normalized().into_owned())
        .map_err(|_| internal.uri.span().unstable().error("invalid route URI"))
}

fn explode<I>(route_str: &str, items: I) -> TokenStream2
    where I: Iterator<Item = (Ident, Type, Expr)>
{
    // Generate the statements to typecheck each parameter.
    // Building <$T as ::rocket::http::uri::FromUriParam<_>>::from_uri_param($e).
    let mut let_bindings = vec![];
    let mut fmt_exprs = vec![];

    for (mut ident, ty, expr) in items {
        let (span, mut expr) = (expr.span(), expr.clone());
        ident.set_span(span);
        let ident_tmp = ident.prepend("tmp");

        let_bindings.push(quote_spanned!(span =>
            let #ident_tmp = #expr; let #ident = <#ty as ::rocket::http::uri::FromUriParam<_>>::from_uri_param(#ident_tmp);
        ));

        // generating: arg tokens for format string
        fmt_exprs.push(quote_spanned!(span => { &#ident as &::rocket::http::uri::UriDisplay }));
    }

    // Convert all of the '<...>' into '{}'.
    let mut inside = false;
    let fmt_string: String = route_str.chars().filter_map(|c| {
        Some(match c {
            '<' => { inside = true; '{' }
            '>' => { inside = false; '}' }
            _ if !inside => c,
            _ => return None
        })
    }).collect();

    // Don't allocate if there are no formatting expressions.
    if fmt_exprs.is_empty() {
        quote!({ #fmt_string.into() })
    } else {
        quote!({ #(#let_bindings)* format!(#fmt_string, #(#fmt_exprs),*).into() })
    }
}

crate fn _uri_internal_macro(input: TokenStream) -> Result<TokenStream> {
    // Parse the internal invocation and the user's URI param expressions.
    let internal = syn::parse::<InternalUriParams>(input).map_err(syn_to_diag)?;
    let exprs = extract_exprs(&internal)?;
    let origin = extract_origin(&internal)?;

    // Determine how many parameters there are in the URI path.
    let path_param_count = origin.path().matches('<').count();

    // Create an iterator over the `ident`, `ty`, and `expr` triple.
    let mut arguments = internal.fn_args
        .into_iter()
        .zip(exprs.into_iter())
        .map(|(FnArg { ident, ty }, expr)| (ident, ty, expr));

    // Generate an expression for both the path and query.
    let path = explode(origin.path(), arguments.by_ref().take(path_param_count));
    let query = if let Some(expr) = origin.query().map(|q| explode(q, arguments)) {
        quote!({ Some(#expr) })
    } else {
        quote!({ None })
    };

    Ok(quote!({
        ::rocket::http::uri::Origin::new::<
                                   ::std::borrow::Cow<'static, str>,
                                   ::std::borrow::Cow<'static, str>,
                                 >(#path, #query)
    }).into())
}
