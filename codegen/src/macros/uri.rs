extern crate syntax_pos;

use std::fmt::Display;

use URI_INFO_MACRO_PREFIX;
use super::prefix_path;

use parser::{UriParams, InternalUriParams, Validation};

use syntax::codemap::Span;
use syntax::ext::base::{DummyResult, ExtCtxt, MacEager, MacResult};
use syntax::tokenstream::{TokenStream, TokenTree};
use syntax::ast::{self, Ident};
use syntax::parse::PResult;
use syntax::ext::build::AstBuilder;
use syntax::ptr::P;

/// FIXME: Implement `MultiSpan::From<Vec<Span>>`.
use self::syntax_pos::MultiSpan as MS;

pub fn uri(
    ecx: &mut ExtCtxt,
    sp: Span,
    args: &[TokenTree],
) -> Box<MacResult + 'static> {
    // Generate the path to the internal macro.
    let mut parser = ecx.new_parser_from_tts(args);
    let (_, mut macro_path) = try_parse!(sp, UriParams::parse_prelude(&mut parser));
    prefix_path(URI_INFO_MACRO_PREFIX, &mut macro_path);

    // It's incredibly important we use `sp` as the Span for the generated code
    // so that errors from the `internal` call show up on the user's code.
    let expr = parser.mk_mac_expr(sp,
        ast::Mac_ {
            path: macro_path,
            tts: args.to_vec().into_iter().collect::<TokenStream>().into(),
        },
        ::syntax::util::ThinVec::new(),
    );

    MacEager::expr(expr)
}

fn extract_exprs<'a>(
    ecx: &ExtCtxt<'a>,
    internal: &InternalUriParams,
) -> PResult<'a, Vec<P<ast::Expr>>> {
    let route_name = &internal.uri_params.route_path;
    match internal.validate() {
        Validation::Ok(exprs) => Ok(exprs),
        Validation::Unnamed(expected, actual) => {
            let mut diag = ecx.struct_span_err(internal.uri_params.args_span(),
                &format!("`{}` route uri expects {} but {} supplied",
                         route_name, p!(expected, "parameter"), p!(actual, "was")));

            if expected > 0 {
                let ps = p!("parameter", expected);
                diag.note(&format!("expected {}: {}", ps, internal.fn_args_str()));
            }

            Err(diag)
        }
        Validation::Named(missing, extra, dup) => {
            let e = &format!("invalid parameters for `{}` route uri", route_name);
            let mut diag = ecx.struct_span_err(internal.uri_params.args_span(), e);
            diag.note(&format!("uri parameters are: {}", internal.fn_args_str()));

            fn join<S: Display, T: Iterator<Item = S>>(iter: T) -> (&'static str, String) {
                let items: Vec<_> = iter.map(|i| format!("`{}`", i)).collect();
                (p!("parameter", items.len()), items.join(", "))
            }

            if !extra.is_empty() {
                let (ps, msg) = join(extra.iter().map(|id| id.node));
                let sp: Vec<_> = extra.iter().map(|ident| ident.span).collect();
                diag.span_help(MS::from_spans(sp), &format!("unknown {}: {}", ps, msg));
            }

            if !dup.is_empty() {
                let (ps, msg) = join(dup.iter().map(|id| id.node));
                let sp: Vec<_> = dup.iter().map(|ident| ident.span).collect();
                diag.span_help(MS::from_spans(sp), &format!("duplicate {}: {}", ps, msg));
            }

            if !missing.is_empty() {
                let (ps, msg) = join(missing.iter());
                diag.help(&format!("missing {}: {}", ps, msg));
            }

            Err(diag)
        }
    }
}

#[allow(unused_imports)]
pub fn uri_internal(
    ecx: &mut ExtCtxt,
    sp: Span,
    tt: &[TokenTree],
) -> Box<MacResult + 'static> {
    // Parse the internal invocation and the user's URI param expressions.
    let mut parser = ecx.new_parser_from_tts(tt);
    let internal = try_parse!(sp, InternalUriParams::parse(ecx, &mut parser));
    let exprs = try_parse!(sp, extract_exprs(ecx, &internal));

    // Generate the statements to typecheck each parameter. First, the mount.
    let mut argument_stmts = vec![];
    let mut format_assign_tokens = vec![];
    let mut fmt_string = internal.uri.node.replace('<', "{").replace('>', "}");
    if let Some(mount_point) = internal.uri_params.mount_point {
        // TODO: Should all expressions, not just string literals, be allowed?
        // let as_ref = ecx.expr_method_call(span, expr, Ident::from_str("as_ref"), v![]);
        let mount_string = mount_point.node;
        argument_stmts.push(ecx.stmt_let_typed(
            mount_point.span,
            false,
            Ident::from_str("mount"),
            quote_ty!(ecx, &str),
            quote_expr!(ecx, $mount_string),
        ));

        format_assign_tokens.push(quote_tokens!(ecx, mount = mount,));
        fmt_string = "{mount}".to_string() + &fmt_string;
    }

    // Now the user's parameters.
    for (i, &(ident, ref ty)) in internal.fn_args.iter().enumerate() {
        let (span, expr) = (exprs[i].span, exprs[i].clone());
        let into = ecx.expr_method_call(span, expr, Ident::from_str("into"), vec![]);
        let stmt = ecx.stmt_let_typed(span, false, ident.node, ty.clone(), into);

        argument_stmts.push(stmt);
        format_assign_tokens.push(quote_tokens!(ecx,
            $ident = &$ident as &::rocket::http::uri::UriDisplay,
        ));
    }

    MacEager::expr(quote_expr!(ecx, {
        $argument_stmts
        ::rocket::http::uri::URI::from(format!($fmt_string, $format_assign_tokens))
    }))
}
