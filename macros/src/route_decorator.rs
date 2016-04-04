use super::{STRUCT_PREFIX, FN_PREFIX};
use utils::*;

use std::str::FromStr;
use std::collections::HashSet;

use syntax::ext::quote::rt::ToTokens;
use syntax::codemap::{Span, BytePos, /* DUMMY_SP, */ Spanned};
use syntax::ast::{Ident, TokenTree, PatKind, Stmt};
use syntax::ast::{Item, Expr, ItemKind, MetaItem, MetaItemKind, FnDecl, Ty};
use syntax::ext::base::{Annotatable, ExtCtxt};
use syntax::ptr::P;
use syntax::ext::build::AstBuilder;
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

fn get_fn_decl<'a>(ecx: &mut ExtCtxt, sp: Span, annotated: &'a Annotatable)
        -> (&'a P<Item>, Spanned<&'a FnDecl>) {
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

    (item, wrap_span(&*fn_decl, item.span))
}

fn get_route_params(ecx: &mut ExtCtxt, meta_item: &MetaItem) -> Params {
    // First, check that the macro was used in the #[route(a, b, ..)] form.
    let params: &Vec<P<MetaItem>> = match meta_item.node {
        MetaItemKind::List(_, ref params) => params,
        _ => ecx.span_fatal(meta_item.span,
                   "Incorrect use of macro. correct form is: #[route(...)]"),
    };

    // Ensure we can unwrap the k = v params.
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
    let path = kv_pairs.get("path").map_or(dummy_kvspan("/".to_string()), |s| {
        s.clone().map(|str_string| String::from(str_string))
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

        if f.node.chars().filter(|c| *c == '<').count() != 1 {
            ecx.span_err(f.p_span, "`form` must contain exactly one parameter");
        }

        Some(f.clone().map(|str_string| String::from(str_string)))
    });

    Params {
        method: method,
        path: path,
        form: form
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

// TODO: Put something like this in the library. Maybe as an iterator?
pub fn gen_params_hashset<'a>(ecx: &ExtCtxt, params: &Spanned<&'a str>)
        -> HashSet<&'a str> {
    let mut seen = HashSet::new();
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

                let mut param_span = params.span.clone();
                param_span.lo = params.span.lo + BytePos(start as u32);
                param_span.hi = params.span.lo + BytePos((i + 1) as u32);

                if i > start + 1 {
                    let param_name = &params.node[(start + 1)..i];
                    if seen.contains(param_name) {
                        let msg = format!("\"{}\" appears more than once in \
                            the parameter string.", param_name);
                        ecx.span_err(param_span, msg.as_str());
                    }

                    seen.insert(param_name);
                } else {
                    ecx.span_err(param_span, "Parameter names cannot be empty.");
                }
            },
            '<' if matching => ecx.span_err(params.span, bad_match_err),
            '>' if !matching => ecx.span_err(params.span, bad_match_err),
            _ => { /* ... */ }
        }
    }

    seen
}

#[derive(Debug)]
struct SimpleArg {
    name: String,
    ty: P<Ty>,
    span: Span
}

pub fn gen_kv_string_hashset<'a>(ecx: &ExtCtxt, params: &'a KVSpanned<String>)
        -> HashSet<&'a str> {
    let mut param_span = params.v_span.clone();
    param_span.lo = params.v_span.lo + BytePos(1);
    let params = Spanned {
        span: param_span,
        node: &*params.node
    };

    gen_params_hashset(ecx, &params)
}

impl SimpleArg {
    fn new<T: ToString>(name: T, ty: P<Ty>, sp: Span) -> SimpleArg {
        SimpleArg { name: name.to_string(), ty: ty, span: sp }
    }

    fn as_str(&self) -> &str {
        self.name.as_str()
    }
}

impl ToTokens for SimpleArg {
    fn to_tokens(&self, cx: &ExtCtxt) -> Vec<TokenTree> {
        str_to_ident(self.as_str()).to_tokens(cx)
    }
}

fn get_fn_params<'a>(ecx: &ExtCtxt, dec_span: Span, path: &'a KVSpanned<String>,
        fn_decl: &Spanned<&FnDecl>, mut external: HashSet<&'a str>)
            -> Vec<SimpleArg> {
    debug!("FUNCTION: {:?}", fn_decl);
    let mut path_params = gen_kv_string_hashset(ecx, &path);

    // Ensure that there are no collisions between path parameters and external
    // params. If there are, get rid of one of them so we don't double error.
    let new_external = external.clone();
    for param in path_params.intersection(&new_external) {
        let msg = format!("'{}' appears as a parameter more than once.", param);
        external.remove(param);
        ecx.span_err(dec_span, msg.as_str());
    }

    // Ensure every param in the function declaration is in `path`. Also add
    // each param name in the declaration to the result vector.
    let mut result = vec![];
    for arg in &fn_decl.node.inputs {
        let ident: &Ident = match arg.pat.node {
            PatKind::Ident(_, ref ident, _) => &ident.node,
            _ => {
                ecx.span_err(arg.pat.span, "Expected an identifier.");
                return result
            }
        };

        let name = ident.to_string();
        if !path_params.remove(name.as_str()) && !external.remove(name.as_str()) {
            let msg1 = format!("'{}' appears in the function declaration...", name);
            let msg2 = format!("...but does not appear as a parameter \
                         (e.g., <{}>).", name);
            ecx.span_err(arg.pat.span, msg1.as_str());
            ecx.span_err(dec_span, msg2.as_str());
        }

        result.push(SimpleArg::new(name, arg.ty.clone(), arg.pat.span.clone()));
    }

    // Ensure every param in `path` and `exclude` is in the function declaration.
    for item in path_params {
            let msg = format!("'{}' appears in the path string...", item);
            ecx.span_err(path.v_span, msg.as_str());
            ecx.span_err(fn_decl.span, "...but does not appear in the function \
                         declration.");
    }

    // FIXME: need the spans for the external params
    for item in external {
            let msg = format!("'{}' appears as a parameter...", item);
            ecx.span_err(dec_span, msg.as_str());
            ecx.span_err(fn_decl.span, "...but does not appear in the function \
                         declaration.");
    }

    result
}

fn get_form_stmt(ecx: &ExtCtxt, fn_args: &mut Vec<SimpleArg>,
                 form_params: &HashSet<&str>) -> Option<Stmt> {
    if form_params.len() < 1 {
        return None
    } else if form_params.len() > 1 {
        panic!("Allowed more than 1 form parameter!");
    }


    let param_ty;
    let param_ident;
    let param_name = form_params.iter().next().unwrap();

    {
        // Get the first item in the hashset, i.e., the form params variable name.
        let fn_arg = fn_args.iter().filter(|a| &&*a.name == param_name).next();
        if fn_arg.is_none() {
            // This happens when a form parameter doesn't appear in the function.
            return None;
        }

        param_ty = fn_arg.unwrap().ty.clone();
        param_ident = str_to_ident(param_name);
    }

    debug!("Form parameter variable: {}: {:?}", param_name, param_ty);
    let fn_arg_index = fn_args.iter().position(|a| &&*a.name == param_name).unwrap();
    fn_args.remove(fn_arg_index);
    quote_stmt!(ecx,
        // TODO: Actually get the form parameters to pass into from_form_string.
        // Alternatively, pass in some already parsed thing.
        let $param_ident: $param_ty = {
            let form_string = std::str::from_utf8(_req.data);
            if form_string.is_err() {
                return rocket::Response::not_found()
            };

            match FromForm::from_form_string(form_string.unwrap()) {
                Ok(v) => v,
                Err(_) => {
                    println!("\t=> Form failed to parse.");
                    return rocket::Response::not_found()
                }
            }
        }
    )
}

// FIXME: Compilation fails when parameters have the same name as the function!
pub fn route_decorator(ecx: &mut ExtCtxt, sp: Span, meta_item: &MetaItem,
          annotated: &Annotatable, push: &mut FnMut(Annotatable)) {
    let (item, fn_decl) = get_fn_decl(ecx, sp, annotated);
    let route_params = get_route_params(ecx, meta_item);

    // TODO: move this elsewhere
    let mut external_params = HashSet::new();
    let mut form_param_hashset = HashSet::new();
    if let Some(ref form) = route_params.form {
        form_param_hashset = gen_kv_string_hashset(ecx, form);
        external_params.extend(&form_param_hashset);
    }

    let mut fn_params = get_fn_params(ecx, sp, &route_params.path, &fn_decl,
                                      external_params.clone());

    // Create a comma seperated list (token tree) of the function parameters
    // We pass this in to the user's function that we're wrapping.
    let fn_param_idents = token_separate(ecx, &fn_params, token::Comma);

    // Generate the statements that will attempt to parse forms during run-time.
    // let form_span = route_params.form.map_or(DUMMY_SP, |f| f.span.clone());
    let form_stmt = get_form_stmt(ecx, &mut fn_params, &mut form_param_hashset);
    form_stmt.as_ref().map(|s| debug!("Form stmt: {:?}", stmt_to_string(s)));

    // Generate the statements that will attempt to parse the paramaters during
    // run-time.
    let mut fn_param_exprs = vec![];
    for (i, param) in fn_params.iter().enumerate() {
        let param_ident = str_to_ident(param.as_str());
        let param_ty = &param.ty;
        let param_fn_item = quote_stmt!(ecx,
            let $param_ident: $param_ty = match _req.get_param($i) {
                Ok(v) => v,
                Err(_) => return rocket::Response::not_found()
            };
        ).unwrap();

        debug!("Param FN: {:?}", stmt_to_string(&param_fn_item));
        fn_param_exprs.push(param_fn_item);
    }

    debug!("Final Params: {:?}", fn_params);
    let route_fn_name = prepend_ident(FN_PREFIX, &item.ident);
    let fn_name = item.ident;
    let route_fn_item = quote_item!(ecx,
         fn $route_fn_name<'rocket>(_req: rocket::Request<'rocket>)
                -> rocket::Response<'rocket> {
             $form_stmt
             $fn_param_exprs
             let result = $fn_name($fn_param_idents);
             rocket::Response::new(result)
         }
    ).unwrap();

    debug!("{}", item_to_string(&route_fn_item));
    push(Annotatable::Item(route_fn_item));

    let struct_name = prepend_ident(STRUCT_PREFIX, &item.ident);
    let path = route_params.path.node;
    let method = method_variant_to_expr(ecx, route_params.method.node);
    push(Annotatable::Item(quote_item!(ecx,
        #[allow(non_upper_case_globals)]
        pub static $struct_name: rocket::StaticRouteInfo = rocket::StaticRouteInfo {
            method: $method,
            path: $path,
            handler: $route_fn_name
        };
    ).unwrap()));
}

