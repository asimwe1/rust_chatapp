use std::collections::HashSet;

use ::{ROUTE_STRUCT_PREFIX, ROUTE_FN_PREFIX};
use utils::{emit_item, span, sep_by_tok, SpanExt, IdentExt, ArgExt, option_as_expr};
use parser::RouteParams;

use syntax::codemap::{Span, Spanned};
use syntax::tokenstream::TokenTree;
use syntax::ast::{Arg, Ident, Stmt, Expr, MetaItem, Path};
use syntax::ext::base::{Annotatable, ExtCtxt};
use syntax::ext::build::AstBuilder;
use syntax::parse::token::{self, str_to_ident};
use syntax::ptr::P;

use rocket::{Method, ContentType};
use rocket::content_type::{TopLevel, SubLevel};

fn method_variant_to_expr(ecx: &ExtCtxt, method: Method) -> Path {
    quote_enum!(ecx, method => ::rocket::Method {
        Options, Get, Post, Put, Delete, Head, Trace, Connect, Patch;
    })
}

fn top_level_to_expr(ecx: &ExtCtxt, level: &TopLevel) -> Path {
    quote_enum!(ecx, *level => ::rocket::content_type::TopLevel {
        Star, Text, Image, Audio, Video, Application, Multipart, Model, Message;
        Ext(ref s) => quote_path!(ecx, ::rocket::content_type::TopLevel::Ext($s))
    })
}

fn sub_level_to_expr(ecx: &ExtCtxt, level: &SubLevel) -> Path {
    quote_enum!(ecx, *level => ::rocket::content_type::SubLevel {
        Star, Plain, Html, Xml, Javascript, Css, EventStream, Json,
        WwwFormUrlEncoded, Msgpack, OctetStream, FormData, Png, Gif, Bmp, Jpeg;
        Ext(ref s) => quote_path!(ecx, ::rocket::content_type::SubLevel::Ext($s))
    })
}

fn content_type_to_expr(ecx: &ExtCtxt, ct: Option<ContentType>) -> Option<P<Expr>> {
    ct.map(|ct| {
        let top_level = top_level_to_expr(ecx, &ct.0);
        let sub_level = sub_level_to_expr(ecx, &ct.1);
        quote_expr!(ecx, ::rocket::ContentType($top_level, $sub_level, None))
    })
}

trait RouteGenerateExt {
    fn generate_form_statement(&self, ecx: &ExtCtxt) -> Option<Stmt>;
    fn generate_param_statements(&self, ecx: &ExtCtxt) -> Vec<Stmt>;
    fn generate_fn_arguments(&self, ecx: &ExtCtxt) -> Vec<TokenTree>;
    fn explode(&self, ecx: &ExtCtxt) -> (&String, Path, P<Expr>, P<Expr>);
}

impl RouteGenerateExt for RouteParams {
    fn generate_form_statement(&self, ecx: &ExtCtxt) -> Option<Stmt> {
        let param = self.form_param.as_ref();
        let arg = param.and_then(|p| self.annotated_fn.find_input(p.value()));
        if param.is_none() {
            return None;
        } else if arg.is_none() {
            let param = param.unwrap();
            let fn_span = self.annotated_fn.span();
            let msg = format!("'{}' is declared as an argument...", param.value());
            ecx.span_err(param.span, &msg);
            ecx.span_err(fn_span, "...but isn't in the function signature.");
            return None;
        }

        let arg = arg.unwrap();
        let (name, ty) = (arg.ident().unwrap(), &arg.ty);
        Some(quote_stmt!(ecx,
            let $name: $ty =
                if let Ok(s) = ::std::str::from_utf8(_req.data.as_slice()) {
                    if let Ok(v) = ::rocket::form::FromForm::from_form_string(s) {
                        v
                    } else {
                        return ::rocket::Response::not_found();
                    }
                } else {
                    return ::rocket::Response::server_error();
                };
        ).expect("form statement"))
    }

    // TODO: Add some kind of logging facility in Rocket to get be able to log
    // an error/debug message if parsing a parameter fails.
    fn generate_param_statements(&self, ecx: &ExtCtxt) -> Vec<Stmt> {
        let mut fn_param_statements = vec![];

        // Retrieve an iterator over the user's path parameters and ensure that
        // each parameter appears in the function signature.
        for param in self.path_params(ecx) {
            if self.annotated_fn.find_input(param.node).is_none() {
                let fn_span = self.annotated_fn.span();
                let msg = format!("'{}' is declared as an argument...", param.node);
                ecx.span_err(param.span, &msg);
                ecx.span_err(fn_span, "...but isn't in the function signature.");
            }
        }

        // Create a function thats checks if an argument was declared in `path`.
        let set: HashSet<&str> = self.path_params(ecx).map(|p| p.node).collect();
        let declared = &|arg: &&Arg| set.contains(&*arg.name().unwrap());

        // These are all of the arguments in the function signature.
        let all = &self.annotated_fn.decl().inputs;

        // Generate code for each user declared parameter.
        for (i, arg) in all.iter().filter(declared).enumerate() {
            let (ident, ty) = (arg.ident().unwrap(), &arg.ty);
            fn_param_statements.push(quote_stmt!(ecx,
                let $ident: $ty = match _req.get_param($i) {
                    Ok(v) => v,
                    Err(_) => return ::rocket::Response::forward()
                };
            ).expect("declared param parsing statement"));
        }

        // A from_request parameter is one that isnt't declared and isn't `form`.
        let from_request = |a: &&Arg| {
            let a_name = &*a.name().unwrap();
            !declared(a)
                && self.form_param.as_ref().map_or(true, |p| p.value() != a_name)
        };

        // Generate the code for `form_request` parameters.
        for arg in all.iter().filter(from_request) {
            let (ident, ty) = (arg.ident().unwrap(), &arg.ty);
            fn_param_statements.push(quote_stmt!(ecx,
                let $ident: $ty = match
                <$ty as ::rocket::request::FromRequest>::from_request(&_req) {
                    Ok(v) => v,
                    Err(_e) => return ::rocket::Response::forward()
                };
            ).expect("undeclared param parsing statement"));
        }

        fn_param_statements
    }

    fn generate_fn_arguments(&self, ecx: &ExtCtxt) -> Vec<TokenTree> {
        let args = self.annotated_fn.decl().inputs.iter().map(|a| {
            a.ident().expect("function decl pat -> ident").clone()
        }).collect::<Vec<Ident>>();

        sep_by_tok(ecx, &args, token::Comma)
    }

    fn explode(&self, ecx: &ExtCtxt) -> (&String, Path, P<Expr>, P<Expr>) {
        let path = &self.path.node;
        let method = method_variant_to_expr(ecx, self.method.node);
        let format = self.format.as_ref().map(|kv| kv.value().clone());
        let content_type = option_as_expr(ecx, &content_type_to_expr(ecx, format));
        let rank = option_as_expr(ecx, &self.rank);

        (path, method, content_type, rank)
    }
}

// FIXME: Compilation fails when parameters have the same name as the function!
fn generic_route_decorator(known_method: Option<Spanned<Method>>,
                           ecx: &mut ExtCtxt,
                           sp: Span,
                           meta_item: &MetaItem,
                           annotated: &Annotatable,
                           push: &mut FnMut(Annotatable)) {
    // Initialize the logger.
    ::rocket::logger::init(::rocket::LoggingLevel::Debug);

    // Parse the route and generate the code to create the form and param vars.
    let route = RouteParams::from(ecx, sp, known_method, meta_item, annotated);
    let form_statement = route.generate_form_statement(ecx);
    let param_statements = route.generate_param_statements(ecx);
    let fn_arguments = route.generate_fn_arguments(ecx);

    // Generate and emit the wrapping function with the Rocket handler signature.
    let user_fn_name = route.annotated_fn.ident();
    let route_fn_name = user_fn_name.prepend(ROUTE_FN_PREFIX);
    emit_item(push, quote_item!(ecx,
         fn $route_fn_name<'rocket>(_req: &'rocket ::rocket::Request<'rocket>)
                -> ::rocket::Response<'rocket> {
             $form_statement
             $param_statements
             let result = $user_fn_name($fn_arguments);
             ::rocket::Response::new(result)
         }
    ).unwrap());

    // Generate and emit the static route info that uses the just generated
    // function as its handler. A proper Rocket route will be created from this.
    let struct_name = user_fn_name.prepend(ROUTE_STRUCT_PREFIX);
    let (path, method, content_type, rank) = route.explode(ecx);
    emit_item(push, quote_item!(ecx,
        #[allow(non_upper_case_globals)]
        pub static $struct_name: ::rocket::StaticRouteInfo =
            ::rocket::StaticRouteInfo {
                method: $method,
                path: $path,
                handler: $route_fn_name,
                format: $content_type,
                rank: $rank,
            };
    ).unwrap());
}

pub fn route_decorator(ecx: &mut ExtCtxt,
                       sp: Span,
                       meta_item: &MetaItem,
                       annotated: &Annotatable,
                       push: &mut FnMut(Annotatable)) {
    generic_route_decorator(None, ecx, sp, meta_item, annotated, push);
}

macro_rules! method_decorator {
    ($name:ident, $method:ident) => (
        pub fn $name(ecx: &mut ExtCtxt, sp: Span, meta_item: &MetaItem,
                     annotated: &Annotatable, push: &mut FnMut(Annotatable)) {
            let i_sp = meta_item.span.shorten_to(meta_item.name().len() as u32);
            let method = Some(span(Method::$method, i_sp));
            generic_route_decorator(method, ecx, sp, meta_item, annotated, push);
        }
    )
}

method_decorator!(get_decorator, Get);
method_decorator!(put_decorator, Put);
method_decorator!(post_decorator, Post);
method_decorator!(delete_decorator, Delete);
method_decorator!(patch_decorator, Patch);
