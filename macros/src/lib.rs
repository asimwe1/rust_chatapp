#![crate_type = "dylib"]
#![feature(quote, concat_idents, plugin_registrar, rustc_private)]

#[macro_use] extern crate syntax;
extern crate rustc;
extern crate rustc_plugin;
extern crate rocket;

#[macro_use] mod macro_utils;

use macro_utils::{prepend_ident, get_key_values};

use std::str::FromStr;
use rustc_plugin::Registry;

use syntax::parse::token::{intern};
use syntax::ext::base::SyntaxExtension;
use syntax::ext::quote::rt::ToTokens;
use syntax::codemap::Span;
use syntax::ast::{Item, ItemKind, MetaItem, MetaItemKind, FnDecl, LitKind};
use syntax::ext::base::{Annotatable, ExtCtxt};
use syntax::ptr::P;

use syntax::ast::{Path, PathSegment, Expr, ExprKind, TokenTree};
use syntax::ext::base::{DummyResult, MacResult, MacEager};
use syntax::ext::build::AstBuilder;  // trait for expr_usize
use syntax::parse::parser::{Parser, PathParsingMode};
use syntax::parse::PResult;
use syntax::parse::token::Token;

const DEBUG: bool = true;

const STRUCT_PREFIX: &'static str = "ROCKET_ROUTE_STRUCT_";
const FN_PREFIX: &'static str = "rocket_route_fn_";

use rocket::Method;

struct Params {
    method: Method,
    path: String
}

fn bad_item_fatal(ecx: &mut ExtCtxt, dec_sp: Span, i_sp: Span) -> ! {
    ecx.span_err(dec_sp, "This decorator cannot be used on non-functions...");
    ecx.span_fatal(i_sp, "...but an attempt to use it on the item below was made.")
}

fn bad_method_err(ecx: &mut ExtCtxt, dec_sp: Span, message: &str) -> Method {
    let message = format!("{} Valid methods are: [GET, PUT, POST, DELETE, \
        OPTIONS, HEAD, TRACE, CONNECT, PATCH]", message);
    ecx.span_err(dec_sp, message.as_str());
    Method::Get
}

fn get_fn_decl<'a>(ecx: &mut ExtCtxt, sp: Span, annotated: &'a Annotatable)
        -> (&'a P<Item>, &'a P<FnDecl>) {
    // `annotated` is the AST object for the annotated item.
    let item: &P<Item> = match annotated {
        &Annotatable::Item(ref item) => item,
        &Annotatable::TraitItem(ref item) => bad_item_fatal(ecx, sp, item.span),
        &Annotatable::ImplItem(ref item) => bad_item_fatal(ecx, sp, item.span)
    };

    let fn_decl: &P<FnDecl> = match item.node {
        ItemKind::Fn(ref decl, _, _, _, _, _) => decl,
        _ => bad_item_fatal(ecx, sp, item.span)
    };

    (item, fn_decl)
}

fn get_route_params(ecx: &mut ExtCtxt, meta_item: &MetaItem) -> Params {
    // First, check that the macro was used in the #[route(a, b, ..)] form.
    let params: &Vec<P<MetaItem>> = match meta_item.node {
        MetaItemKind::List(_, ref params) => params,
        _ => ecx.span_fatal(meta_item.span,
                   "incorrect use of macro. correct form is: #[demo(...)]"),
    };

    // Ensure we can unwrap the k = v params.
    if params.len() < 1 {
        bad_method_err(ecx, meta_item.span, "HTTP method parameter is missing.");
        ecx.span_fatal(meta_item.span, "At least 2 arguments are required.");
    }

    // Get the method and the rest of the k = v params. Ensure method parameter
    // is valid. If it's not, issue an error but use "GET" to continue parsing.
    let (method_param, kv_params) = params.split_first().unwrap();
    let method = if let MetaItemKind::Word(ref word) = method_param.node {
        Method::from_str(word).unwrap_or_else(|_| {
            let message = format!("{} is not a valid method.", word);
            bad_method_err(ecx, method_param.span, message.as_str())
        })
    } else {
        bad_method_err(ecx, method_param.span, "Invalid parameter. Expected a
            valid HTTP method at this position.")
    };

    // Now grab all of the required and optional parameters.
    let req: [&'static str; 1] = ["path"];
    let opt: [&'static str; 0] = [];
    let kv_pairs = get_key_values(ecx, meta_item.span, &req, &opt, kv_params);

    // Ensure we have a path, just to keep parsing and generating errors.
    let path = kv_pairs.get("path").map_or(String::from("/"), |s| {
        String::from(*s)
    });

    Params {
        method: method,
        path: path
    }
}

fn route_decorator(ecx: &mut ExtCtxt, sp: Span, meta_item: &MetaItem,
          annotated: &Annotatable, push: &mut FnMut(Annotatable)) {
    let (item, fn_decl) = get_fn_decl(ecx, sp, annotated);
    let route_params = get_route_params(ecx, meta_item);

    let route_fn_name = prepend_ident(FN_PREFIX, &item.ident);
    let fn_name = item.ident;
    push(Annotatable::Item(quote_item!(ecx,
         fn $route_fn_name(_req: Request) -> Response {
             let result = $fn_name();
             println!("Routing function. Result: {}", result);
             Response
         }
    ).unwrap()));

    let struct_name = prepend_ident(STRUCT_PREFIX, &item.ident);
    let path = route_params.path;
    let struct_item = quote_item!(ecx,
        #[allow(non_upper_case_globals)]
        pub static $struct_name: Route<'static> = Route {
            method: Method::Get, // FIXME
            path: $path,
            handler: $route_fn_name
        };
    ).unwrap();
    push(Annotatable::Item(struct_item));
}

fn get_paths<'a>(parser: &mut Parser<'a>) -> PResult<'a, Vec<Path>> {
    if parser.eat(&Token::Eof) {
        return Ok(vec![]);
    }

    let mut results = Vec::new();
    loop {
        results.push(try!(parser.parse_path(PathParsingMode::NoTypesAllowed)));
        if !parser.eat(&Token::Comma) {
            try!(parser.expect(&Token::Eof));
            break;
        }
    }

    Ok(results)
}

fn routes_macro(ecx: &mut ExtCtxt, sp: Span, args: &[TokenTree])
        -> Box<MacResult + 'static> {
    let mut parser = ecx.new_parser_from_tts(args);
    let mut paths = get_paths(&mut parser).unwrap_or_else(|mut e| {
        e.emit();
        vec![]
    });

    // Prefix each path terminator with STRUCT_PREFIX.
    for p in &mut paths {
        let last = p.segments.len() - 1;
        let last_seg = &mut p.segments[last];
        let new_ident = prepend_ident(STRUCT_PREFIX, &last_seg.identifier);
        last_seg.identifier = new_ident;
    }

    // Build up the P<Expr> for each &path.
    let path_exprs = paths.iter().map(|p| { quote_expr!(ecx, &$p) }).collect();
    MacEager::expr(ecx.expr_vec_slice(sp, path_exprs))
}

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_syntax_extension(intern("route"),
        SyntaxExtension::MultiDecorator(Box::new(route_decorator)));
    reg.register_macro("routes", routes_macro);
}
