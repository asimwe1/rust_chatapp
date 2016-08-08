use super::{ROUTE_STRUCT_PREFIX, ROUTE_FN_PREFIX};
use utils::*;

use std::str::FromStr;
use std::collections::HashMap;

use syntax::codemap::{Span, BytePos, /* DUMMY_SP, */ Spanned};
use syntax::ast::{Stmt, Item, Expr, ItemKind, MetaItem, MetaItemKind, FnDecl};
use syntax::ext::base::{Annotatable, ExtCtxt};
use syntax::ptr::P;
use syntax::print::pprust::{item_to_string, stmt_to_string};
use syntax::parse::token::{self, str_to_ident};

use rocket::Method;

#[allow(dead_code)]
const DEBUG: bool = true;

struct Params {
    method: Spanned<Method>,
    path: KVSpanned<String>,
    form: Option<KVSpanned<String>>,
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

pub fn get_fn_decl<'a>(ecx: &mut ExtCtxt, sp: Span, annotated: &'a Annotatable)
        -> (&'a P<Item>, Spanned<&'a FnDecl>) {
    // `annotated` is the AST object for the annotated item.
    let item: &P<Item> = match *annotated {
        Annotatable::Item(ref item) => item,
        Annotatable::TraitItem(ref item) => bad_item_fatal(ecx, sp, item.span),
        Annotatable::ImplItem(ref item) => bad_item_fatal(ecx, sp, item.span)
    };

    let fn_decl: &P<FnDecl> = match item.node {
         ItemKind::Fn(ref decl, _, _, _, _, _) => decl,
         _ => bad_item_fatal(ecx, sp, item.span)
     };

    (item, wrap_span(&*fn_decl, item.span))
}

// Parses the MetaItem derived from the route(...) macro.
fn parse_route(ecx: &mut ExtCtxt, meta_item: &MetaItem) -> Params {
    // Ensure we've been supplied with a k = v meta item. Error out if not.
    let params = meta_item.expect_list(ecx, "Bad use. Expected: #[route(...)]");
    if params.len() < 1 {
        bad_method_err(ecx, meta_item.span, "HTTP method parameter is missing.");
        ecx.span_fatal(meta_item.span, "At least 2 arguments are required.");
    }

    // Get the method and the rest of the k = v params.
    let (method_param, kv_params) = params.split_first().unwrap();

    // Ensure method parameter is valid. If it's not, issue an error but use
    // "GET" to continue parsing. method :: Spanned<Method>.
    let method = if let MetaItemKind::Word(ref word) = method_param.node {
        let method = Method::from_str(word).unwrap_or_else(|_| {
            let message = format!("{} is not a valid method.", word);
            bad_method_err(ecx, method_param.span, message.as_str())
        });

        Spanned { span: method_param.span, node: method }
    } else {
        let method = bad_method_err(ecx, method_param.span, "Invalid parameter. \
            Expected a valid HTTP method at this position.");
        dummy_span(method)
    };

    // Now grab all of the required and optional parameters.
    let req: [&'static str; 1] = ["path"];
    let opt: [&'static str; 1] = ["form"];
    let kv_pairs = get_key_values(ecx, meta_item.span, &req, &opt, kv_params);

    // Ensure we have a path, just to keep parsing and generating errors.
    let path = kv_pairs.get("path").map_or(KVSpanned::dummy("/".to_string()), |s| {
        s.clone().map(String::from)
    });

    // If there's a form parameter, ensure method is POST.
    let form = kv_pairs.get("form").map_or(None, |f| {
        if method.node != Method::Post {
            ecx.span_err(f.p_span, "Use of `form` requires a POST method...");
            let message = format!("...but {} was found instead.", method.node);
            ecx.span_err(method_param.span, message.as_str());
        }

        if !(f.node.starts_with('<') && f.node.ends_with('>')) {
            ecx.struct_span_err(f.p_span, "`form` cannot contain arbitrary text")
                .help("`form` must be exactly one parameter: \"<param>\"")
                .emit();
        }

        if f.node.chars().filter(|c| *c == '<' || *c == '>').count() != 2 {
            ecx.span_err(f.p_span, "`form` must contain exactly one parameter");
        }

        Some(f.clone().map(String::from))
    });

    Params {
        method: method,
        path: path,
        form: form
    }
}

// TODO: Put something like this in the library. Maybe as an iterator?
pub fn extract_params<'a>(ecx: &ExtCtxt, params: &Spanned<&'a str>)
        -> Vec<Spanned<&'a str>> {
    let mut output_params = vec![];
    let bad_match_err = "Parameter string is malformed.";

    let mut start = 0;
    let mut matching = false;
    for (i, c) in params.node.char_indices() {
        match c {
            '<' if !matching => {
                matching = true;
                start = i;
            },
            '>' if matching => {
                matching = false;

                let mut param_span = params.span;
                param_span.lo = params.span.lo + BytePos(start as u32);
                param_span.hi = params.span.lo + BytePos((i + 1) as u32);

                if i > start + 1 {
                    let param_name = &params.node[(start + 1)..i];
                    output_params.push(wrap_span(param_name, param_span))
                } else {
                    ecx.span_err(param_span, "Parameter names cannot be empty.");
                }
            },
            '<' if matching => ecx.span_err(params.span, bad_match_err),
            '>' if !matching => ecx.span_err(params.span, bad_match_err),
            _ => { /* ... */ }
        }
    }

    output_params
}

pub fn extract_params_from_kv<'a>(ecx: &ExtCtxt, params: &'a KVSpanned<String>)
        -> Vec<Spanned<&'a str>> {
    let mut param_span = params.v_span;
    param_span.lo = params.v_span.lo + BytePos(1);
    extract_params(ecx, &Spanned {
        span: param_span,
        node: &*params.node
    })
}

// Analyzes the declared parameters against the function declaration. Returns
// two vectors. The first is the set of parameters declared by the user, and
// the second is the set of parameters not declared by the user.
fn get_fn_params<'a, T: Iterator<Item=&'a Spanned<&'a str>>>(ecx: &ExtCtxt,
        declared_params: T, fn_decl: &Spanned<&FnDecl>)
            -> Vec<UserParam> {
    debug!("FUNCTION: {:?}", fn_decl);

    // First, check that all of the parameters are unique.
    let mut seen: HashMap<&str, &Spanned<&str>> = HashMap::new();
    for item in declared_params {
        if seen.contains_key(item.node) {
            let msg = format!(
                "\"{}\" was declared as a parameter more than once.", item.node);
            ecx.span_err(item.span, msg.as_str());
        } else {
            seen.insert(item.node, item);
        }
    }

    let mut user_params = vec![];

    // Ensure every param in the function declaration was declared by the user.
    for arg in &fn_decl.node.inputs {
        let name = arg.pat.expect_ident(ecx, "Expected identifier.");
        let arg = SimpleArg::new(name, arg.ty.clone(), arg.pat.span);
        if seen.remove(&*name.to_string()).is_some() {
            user_params.push(UserParam::new(arg, true));
        } else {
            user_params.push(UserParam::new(arg, false));
        }
    }

    // Emit an error on every attribute param that didn't match in fn params.
    for item in seen.values() {
        let msg = format!("'{}' was declared in the attribute...", item.node);
        ecx.span_err(item.span, msg.as_str());
        ecx.span_err(fn_decl.span, "...but does not appear in the function \
                     declaration.");
    }

    user_params
}

fn get_form_stmt(ecx: &ExtCtxt, fn_args: &mut Vec<UserParam>,
                 form_params: &[Spanned<&str>]) -> Option<Stmt> {
    if form_params.len() < 1 {
        return None;
    } else if form_params.len() > 1 {
        panic!("Allowed more than 1 form parameter!");
    }

    let param_name = &form_params[0].node;
    let (param_ty, param_ident) = {
        // Get the first item in the hashset, i.e., the form params variable name.
        let fn_arg = fn_args.iter().filter(|a| &&*a.name == param_name).next();
        if fn_arg.is_none() {
            // This happens when a form parameter doesn't appear in the function.
            // We should have already caught this, so just return None.
            return None;
        }

        (fn_arg.unwrap().ty.clone(), str_to_ident(param_name))
    };

    // Remove the paramter from the function arguments.
    debug!("Form parameter variable: {}: {:?}", param_name, param_ty);
    let fn_arg_index = fn_args.iter().position(|a| &&*a.name == param_name).unwrap();
    fn_args.remove(fn_arg_index);

    // The actual code we'll be inserting.
    quote_stmt!(ecx,
        let $param_ident: $param_ty =
            if let Ok(form_string) = ::std::str::from_utf8(_req.data) {
                match ::rocket::form::FromForm::from_form_string(form_string) {
                    Ok(v) => v,
                    Err(_) => {
                        println!("\t=> Form failed to parse.");
                        return ::rocket::Response::not_found();
                    }
                }
            } else {
                return ::rocket::Response::server_error();
            }
    )
}

// Is there a better way to do this? I need something with ToTokens for the
// quote_expr macro that builds the route struct. I tried using
// str_to_ident("rocket::Method::Options"), but this seems to miss the context,
// and you get an 'ident not found' on compile. I also tried using the path expr
// builder from ASTBuilder: same thing.
fn method_variant_to_expr(ecx: &ExtCtxt, method: Method) -> P<Expr> {
    match method {
        Method::Options => quote_expr!(ecx, ::rocket::Method::Options),
        Method::Get => quote_expr!(ecx, ::rocket::Method::Get),
        Method::Post => quote_expr!(ecx, ::rocket::Method::Post),
        Method::Put => quote_expr!(ecx, ::rocket::Method::Put),
        Method::Delete => quote_expr!(ecx, ::rocket::Method::Delete),
        Method::Head => quote_expr!(ecx, ::rocket::Method::Head),
        Method::Trace => quote_expr!(ecx, ::rocket::Method::Trace),
        Method::Connect => quote_expr!(ecx, ::rocket::Method::Connect),
        Method::Patch => quote_expr!(ecx, ::rocket::Method::Patch),
    }
}

// FIXME: Compilation fails when parameters have the same name as the function!
pub fn route_decorator(ecx: &mut ExtCtxt, sp: Span, meta_item: &MetaItem,
          annotated: &Annotatable, push: &mut FnMut(Annotatable)) {
    // Get the encompassing item and function declaration for the annotated func.
    let (item, fn_decl) = get_fn_decl(ecx, sp, annotated);

    // Parse and retrieve all of the parameters of the route.
    let route = parse_route(ecx, meta_item);

    // Get a list of the user declared parameters in `path` and `form`.
    let path_params = extract_params_from_kv(ecx, &route.path);
    let form_thing = route.form.unwrap_or_default(); // Default is empty string.
    let form_params = extract_params_from_kv(ecx, &form_thing);

    // Ensure the params match the function declaration and return the params.
    let all_params = path_params.iter().chain(form_params.iter());
    let mut user_params = get_fn_params(ecx, all_params, &fn_decl);

    // Create a comma seperated list (token tree) of the function parameters
    // We pass this in to the user's function that we're wrapping.
    let fn_param_idents = token_separate(ecx, &user_params, token::Comma);

    // Generate the statements that will attempt to parse forms during run-time.
    // Calling this function also remove the form parameter from fn_params.
    let form_stmt = get_form_stmt(ecx, &mut user_params, &form_params);
    form_stmt.as_ref().map(|s| debug!("Form stmt: {:?}", stmt_to_string(s)));

    // Generate the statements that will attempt to parse the paramaters during
    // run-time.
    let mut fn_param_exprs = vec![];
    for (i, param) in user_params.iter().enumerate() {
        let ident = str_to_ident(param.as_str());
        let ty = &param.ty;
        let param_fn_item =
            if param.declared {
                quote_stmt!(ecx,
                    let $ident: $ty = match _req.get_param($i) {
                        Ok(v) => v,
                        Err(_) => return ::rocket::Response::forward()
                    };
                ).unwrap()
            } else {
                quote_stmt!(ecx,
                    let $ident: $ty = match
                    <$ty as ::rocket::request::FromRequest>::from_request(&_req) {
                        Ok(v) => v,
                        Err(e) => {
                            // TODO: Add $ident and $ty to the string.
                            // TODO: Add some kind of loggin facility in Rocket
                            // to get the formatting right (IE, so it idents
                            // correctly).
                            println!("Failed to parse: {:?}", e);
                            return ::rocket::Response::forward();
                        }
                    };
                ).unwrap()
            };

        debug!("Param FN: {:?}", stmt_to_string(&param_fn_item));
        fn_param_exprs.push(param_fn_item);
    }

    let route_fn_name = prepend_ident(ROUTE_FN_PREFIX, &item.ident);
    let fn_name = item.ident;
    let route_fn_item = quote_item!(ecx,
         fn $route_fn_name<'rocket>(_req: ::rocket::Request<'rocket>)
                -> ::rocket::Response<'rocket> {
             $form_stmt
             $fn_param_exprs
             let result = $fn_name($fn_param_idents);
             ::rocket::Response::new(result)
         }
    ).unwrap();

    debug!("{}", item_to_string(&route_fn_item));
    push(Annotatable::Item(route_fn_item));

    let struct_name = prepend_ident(ROUTE_STRUCT_PREFIX, &item.ident);
    let path = &route.path.node;
    let method = method_variant_to_expr(ecx, route.method.node);
    push(Annotatable::Item(quote_item!(ecx,
        #[allow(non_upper_case_globals)]
        pub static $struct_name: ::rocket::StaticRouteInfo =
            ::rocket::StaticRouteInfo {
                method: $method,
                path: $path,
                handler: $route_fn_name
            };
    ).unwrap()));
}

