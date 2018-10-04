use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use std::fmt::Display;

use derive_utils::{syn, Result};
use syn_ext::{IdentExt, syn_to_diag};

use self::syn::{Expr, Ident, Type};
use self::syn::spanned::Spanned as SynSpanned;
use bang::{prefix_last_segment, uri_parsing::*};

use rocket_http::{uri::Origin, ext::IntoOwned};

const URI_INFO_MACRO_PREFIX: &str = "rocket_uri_for_";

macro_rules! p {
    (@go $num:expr, $singular:expr, $plural:expr) => (
        if $num == 1 { $singular.into() } else { $plural }
    );

    ("parameter", $n:expr) => (p!(@go $n, "parameter", "parameters"));
    ($n:expr, "was") => (p!(@go $n, "1 was", format!("{} were", $n)));
    ($n:expr, "parameter") => (p!(@go $n, "1 parameter", format!("{} parameters", $n)));
}

crate fn _uri_macro(input: TokenStream) -> Result<TokenStream> {
    let input2: TokenStream2 = input.clone().into();
    let mut params = syn::parse::<UriParams>(input).map_err(syn_to_diag)?;
    prefix_last_segment(&mut params.route_path, URI_INFO_MACRO_PREFIX);

    let path = &params.route_path;
    Ok(quote!(#path!(#input2)).into())
}

fn extract_exprs(internal: &InternalUriParams) -> Result<Vec<&Expr>> {
    let route_name = &internal.uri_params.route_path;
    match internal.validate() {
        Validation::Ok(exprs) => Ok(exprs),
        Validation::Unnamed(expected, actual) => {
            let mut diag = internal.uri_params.args_span().error(
                format!("`{}` route uri expects {} but {} supplied", quote!(#route_name),
                         p!(expected, "parameter"), p!(actual, "was")));

            if expected > 0 {
                let ps = p!("parameter", expected);
                diag = diag.note(format!("expected {}: {}", ps, internal.fn_args_str()));
            }

            Err(diag)
        }
        Validation::Named(missing, extra, dup) => {
            let e = format!("invalid parameters for `{}` route uri", quote!(#route_name));
            let mut diag = internal.uri_params.args_span().error(e)
                .note(format!("uri parameters are: {}", internal.fn_args_str()));

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

fn explode<'a, I>(route_str: &str, items: I) -> TokenStream2
    where I: Iterator<Item = (&'a Ident, &'a Type, &'a Expr)>
{
    // Generate the statements to typecheck each parameter.
    // Building <$T as ::rocket::http::uri::FromUriParam<_>>::from_uri_param($e).
    let mut let_bindings = vec![];
    let mut fmt_exprs = vec![];

    for (mut ident, ty, expr) in items {
        let (span, expr) = (expr.span(), expr);
        let ident_tmp = ident.prepend("tmp_");

        let_bindings.push(quote_spanned!(span =>
            let #ident_tmp = #expr;
            let #ident = <#ty as ::rocket::http::uri::FromUriParam<_>>::from_uri_param(#ident_tmp);
        ));

        // generating: arg tokens for format string
        fmt_exprs.push(quote_spanned! { span =>
            &#ident as &::rocket::http::uri::UriDisplay
        });
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
    let mut arguments = internal.fn_args.iter()
        .zip(exprs.iter())
        .map(|(FnArg { ident, ty }, &expr)| (ident, ty, expr));

    // Generate an expression for both the path and query.
    let path = explode(origin.path(), arguments.by_ref().take(path_param_count));

    // FIXME: Use Optional.
    // let query = Optional(origin.query().map(|q| explode(q, arguments)));
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
