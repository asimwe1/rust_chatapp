use super::{ROUTE_STRUCT_PREFIX, ROUTE_FN_PREFIX};
use utils::*;
use meta_item_parser::{MetaItemParser, RouteDecoratorExt};

use std::collections::HashMap;

use syntax::codemap::{Span, BytePos, /* DUMMY_SP, */ Spanned};
use syntax::ast::{Stmt, Expr, MetaItem, FnDecl};
use syntax::ext::base::{Annotatable, ExtCtxt};
use syntax::ptr::P;
use syntax::print::pprust::{item_to_string, stmt_to_string};
use syntax::parse::token::{self, str_to_ident};

use rocket::Method;

#[allow(dead_code)]
const DEBUG: bool = true;

pub fn extract_params_from_kv<'a>(parser: &MetaItemParser,
                    params: &'a KVSpanned<String>) -> Vec<Spanned<&'a str>> {
    let mut param_span = params.v_span;
    param_span.lo = params.v_span.lo + BytePos(1);
    let spanned = span(&*params.node, param_span);
    parser.iter_params(&spanned).collect()
}

// Analyzes the declared parameters against the function declaration. Returns
// a vector of all of the parameters in the order the user wants them.
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
pub fn route_decorator(known_method: Option<Spanned<Method>>, ecx: &mut ExtCtxt,
                       sp: Span, meta_item: &MetaItem, annotated: &Annotatable,
                       push: &mut FnMut(Annotatable)) {
    // Get the encompassing item and function declaration for the annotated func.
    let parser = MetaItemParser::new(ecx, meta_item, annotated, &sp);
    let (item, fn_decl) = (parser.expect_item(), parser.expect_fn_decl());

    // Parse and retrieve all of the parameters of the route.
    let route = parser.parse_route(known_method);

    // Get a list of the user declared parameters in `path` and `form`.
    let path_params = extract_params_from_kv(&parser, &route.path);
    let form_thing = route.form.unwrap_or_default(); // Default is empty string.
    let form_params = extract_params_from_kv(&parser, &form_thing);

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
                handler: $route_fn_name,
                content_type: ::rocket::ContentType(
                    ::rocket::content_type::TopLevel::Star,
                    ::rocket::content_type::SubLevel::Star,
                    None)
            };
    ).unwrap()));
}


pub fn generic_route_decorator(ecx: &mut ExtCtxt,
                       sp: Span, meta_item: &MetaItem, annotated: &Annotatable,
                       push: &mut FnMut(Annotatable)) {
    route_decorator(None, ecx, sp, meta_item, annotated, push);
}

macro_rules! method_decorator {
    ($name:ident, $method:ident) => (
        pub fn $name(ecx: &mut ExtCtxt, sp: Span, meta_item: &MetaItem,
                     annotated: &Annotatable, push: &mut FnMut(Annotatable)) {
            let mut i_sp = meta_item.span;
            i_sp.hi = i_sp.lo + BytePos(meta_item.name().len() as u32);
            let method = Some(span(Method::$method, i_sp));
            route_decorator(method, ecx, sp, meta_item, annotated, push);
        }
    )
}

method_decorator!(get_decorator, Get);
method_decorator!(put_decorator, Put);
method_decorator!(post_decorator, Post);
method_decorator!(delete_decorator, Delete);
method_decorator!(options_decorator, Options);
method_decorator!(head_decorator, Head);
method_decorator!(trace_decorator, Trace);
method_decorator!(connect_decorator, Connect);
method_decorator!(patch_decorator, Patch);
