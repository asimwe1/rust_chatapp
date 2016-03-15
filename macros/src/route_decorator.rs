use super::{STRUCT_PREFIX, FN_PREFIX};
use utils::{prepend_ident, get_key_values};

use std::str::FromStr;
use std::collections::HashSet;

use syntax::ext::quote::rt::ToTokens;
use syntax::codemap::{Span, DUMMY_SP};
use syntax::ast::{Ident, TokenTree, PatKind};
use syntax::ast::{Item, Expr, ItemKind, MetaItem, MetaItemKind, FnDecl};
use syntax::ext::base::{Annotatable, ExtCtxt};
use syntax::ptr::P;
use syntax::ext::build::AstBuilder;
use syntax::print::pprust::item_to_string;
use syntax::parse::token::{self, str_to_ident};

use rocket::Method;

#[allow(dead_code)]
const DEBUG: bool = true;

struct Params {
    method: Method,
    path: String
}

fn bad_item_fatal(ecx: &mut ExtCtxt, dec_sp: Span, i_sp: Span) -> ! {
    ecx.span_err(dec_sp, "This decorator cannot be used on non-functions...");
    ecx.span_fatal(i_sp, "...but it was used on the item below.")
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

// Is there a better way to do this? I need something with ToTokens for the
// quote_expr macro that builds the route struct. I tried using
// str_to_ident("rocket::Method::Options"), but this seems to miss the context,
// and you get an 'ident not found' on compile. I also tried using the path expr
// builder from ASTBuilder: same thing.
fn method_variant_to_expr(ecx: &ExtCtxt, method: Method) -> P<Expr> {
    match method {
        Method::Options => quote_expr!(ecx, rocket::Method::Options),
        Method::Get => quote_expr!(ecx, rocket::Method::Get),
        Method::Post => quote_expr!(ecx, rocket::Method::Post),
        Method::Put => quote_expr!(ecx, rocket::Method::Put),
        Method::Delete => quote_expr!(ecx, rocket::Method::Delete),
        Method::Head => quote_expr!(ecx, rocket::Method::Head),
        Method::Trace => quote_expr!(ecx, rocket::Method::Trace),
        Method::Connect => quote_expr!(ecx, rocket::Method::Connect),
        Method::Patch => quote_expr!(ecx, rocket::Method::Patch),
    }
}

pub fn get_fn_params(ecx: &ExtCtxt, sp: Span, path: &str,
                        fn_decl: &FnDecl) -> Vec<String> {
    let mut seen = HashSet::new();
    let bad_match_err = "Path string is malformed.";
    let mut matching = false;

    // Collect all of the params in the path and insert into HashSet.
    let mut start = 0;
    for (i, c) in path.char_indices() {
        match c {
            '<' if !matching => {
                matching = true;
                start = i;
            },
            '>' if matching => {
                matching = false;
                if start + 1 < i {
                    let param_name = &path[(start + 1)..i];
                    seen.insert(param_name);
                } else {
                    ecx.span_err(sp, "Parameter cannot be empty.");
                }
            },
            '<' if matching => ecx.span_err(sp, bad_match_err),
            '>' if !matching => ecx.span_err(sp, bad_match_err),
            _ => { /* ... */ }
        }
    }

    // Ensure every param in the function declaration is in `path`. Also add
    // each param name in the declaration to the result vector.
    let mut result = vec![];
    for arg in &fn_decl.inputs {
        let ident: &Ident = match arg.pat.node {
            PatKind::Ident(_, ref ident, _) => &ident.node,
            _ => {
                ecx.span_err(sp, "Expected an identifier."); // FIXME: fn span.
                return result
            }
        };

        let name = ident.to_string();
        if !seen.remove(name.as_str()) {
            let msg = format!("'{}' appears in the function declaration but \
                not in the path string.", name);
            ecx.span_err(sp, msg.as_str());
        }

        result.push(name);
    }

    // Ensure every param in `path` is in the function declaration.
    for item in seen {
        let msg = format!("'{}' appears in the path string but not in the \
            function declaration.", item);
        ecx.span_err(sp, msg.as_str());
    }

    result
}

pub fn route_decorator(ecx: &mut ExtCtxt, sp: Span, meta_item: &MetaItem,
          annotated: &Annotatable, push: &mut FnMut(Annotatable)) {
    let (item, fn_decl) = get_fn_decl(ecx, sp, annotated);
    let route_params = get_route_params(ecx, meta_item);
    let fn_params = get_fn_params(ecx, sp, &route_params.path, &fn_decl);

    debug!("Path: {:?}", route_params.path);
    debug!("Function Declaration: {:?}", fn_decl);

    let mut fn_param_exprs = vec![];
    for param in &fn_params {
        let param_ident = str_to_ident(param.as_str());
        fn_param_exprs.push(quote_stmt!(ecx,
            let $param_ident = match _req.get_param($param) {
                Ok(v) => v,
                Err(_) => return rocket::Response::not_found()
            };
        ).unwrap());
    }

    let mut fn_param_idents: Vec<TokenTree> = vec![];
    for i in 0..fn_params.len() {
        let tokens = str_to_ident(fn_params[i].as_str()).to_tokens(ecx);
        fn_param_idents.extend(tokens);
        if i < fn_params.len() - 1 {
            fn_param_idents.push(TokenTree::Token(DUMMY_SP, token::Comma));
        }
    }

    debug!("Final Params: {:?}", fn_params);
    let route_fn_name = prepend_ident(FN_PREFIX, &item.ident);
    let fn_name = item.ident;
    let route_fn_item = quote_item!(ecx,
         fn $route_fn_name<'a>(_req: rocket::Request) -> rocket::Response<'a> {
             $fn_param_exprs
             let result = $fn_name($fn_param_idents);
             rocket::Response::from(result)
         }
    ).unwrap();
    debug!("{}", item_to_string(&route_fn_item));
    push(Annotatable::Item(route_fn_item));

    let struct_name = prepend_ident(STRUCT_PREFIX, &item.ident);
    let path = route_params.path;
    let method = method_variant_to_expr(ecx, route_params.method);
    push(Annotatable::Item(quote_item!(ecx,
        #[allow(non_upper_case_globals)]
        pub static $struct_name: rocket::Route<'static, 'static> = rocket::Route {
            method: $method,
            path: $path,
            handler: $route_fn_name
        };
    ).unwrap()));
}

