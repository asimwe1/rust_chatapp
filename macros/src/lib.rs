#![crate_type = "dylib"]
#![feature(plugin_registrar, rustc_private)]

extern crate syntax;
extern crate rustc;
extern crate rustc_plugin;
extern crate hyper;

use std::str::FromStr;

use rustc_plugin::Registry;
use syntax::parse::token::{intern};
use syntax::ext::base::SyntaxExtension;
use std::default::Default;

use syntax::codemap::Span;
use syntax::ast::{Item, ItemKind, MetaItem, MetaItemKind, FnDecl, LitKind};
use syntax::ext::base::{Annotatable, ExtCtxt};
use syntax::ptr::P;

use hyper::method::Method;

fn bad_item_fatal(ecx: &mut ExtCtxt, dec_sp: Span, item_sp: Span) -> ! {
    ecx.span_err(dec_sp, "This decorator cannot be used on non-functions...");
    ecx.span_fatal(item_sp, "...but an attempt to use it on the item below was made.")
}

fn bad_method_err(ecx: &mut ExtCtxt, dec_sp: Span, method: &str) {
    let message = format!("`{}` is not a valid method. Valid methods are: \
                          [GET, POST, PUT, DELETE, HEAD, PATCH]", method);
    ecx.span_err(dec_sp, message.as_str());
}

struct RouteParams {
    method: Method,
    path: String,
}

fn demo_decorator(ecx: &mut ExtCtxt, sp: Span, meta_item: &MetaItem,
          annotated: &Annotatable, _push: &mut FnMut(Annotatable)) {
    // Word: #[demo]
    // List: #[demo(one, two, ..)] or #[demo(one = "1", ...)] or mix both
    // NameValue: #[demo = "1"]
    let params: &Vec<P<MetaItem>> = match meta_item.node {
        MetaItemKind::List(_, ref params) => params,
        // Would almost certainly be better to use "DummyResult" here.
        _ => ecx.span_fatal(meta_item.span,
                   "incorrect use of macro. correct form is: #[demo(...)]"),
    };

    if params.len() < 2 {
        ecx.span_fatal(meta_item.span, "Bad invocation. Need >= 2 arguments.");
    }

    let (method_param, kv_params) = params.split_first().unwrap();
    let method = if let MetaItemKind::Word(ref word) = method_param.node {
        let method = Method::from_str(word).unwrap_or_else(|e| {
            Method::Extension(String::from(&**word))
        });

        if let Method::Extension(ref name) = method {
            bad_method_err(ecx, meta_item.span, name.as_str());
            Method::Get
        } else {
            method
        }
    } else {
        Method::Get
    };

    let mut route_params: RouteParams = RouteParams {
        method: method,
        path: String::new()
    };

    let mut found_path = false;
    for param in kv_params {
        if let MetaItemKind::NameValue(ref name, ref value) = param.node {
            match &**name {
                "path" => {
                    found_path = true;
                    if let LitKind::Str(ref string, _) = value.node {
                        route_params.path = String::from(&**string);
                    } else {
                        ecx.span_err(param.span, "Path value must be string.");
                    }
                },
                _ => {
                    ecx.span_err(param.span, "Unrecognized parameter.");
                }
            }

        } else {
            ecx.span_err(param.span, "Invalid parameter. Must be key = value.");
        }
    }

    if !found_path {
        ecx.span_err(meta_item.span, "`path` argument is missing.");
    }

    // for param in params {
    //     if let MetaItemKind::Word(ref word) = param.node {
    //         if hyper::method::Method::from_str(word).is_ok() {
    //             println!("METHOD! {}", word);
    //         }

    //         println!("WORD Param: {:?}", param);
    //     } else {
    //         println!("NOT word Param: {:?}", param);
    //     }
    // }

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

    println!("Function arguments: {:?}", fn_decl.inputs);
}

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    // reg.register_macro("rn", expand_rn);
    reg.register_syntax_extension(intern("route"),
        SyntaxExtension::MultiDecorator(Box::new(demo_decorator)));
}
