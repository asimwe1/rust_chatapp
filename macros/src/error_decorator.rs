use utils::*;
use meta_item_parser::MetaItemParser;
use super::{CATCH_STRUCT_PREFIX, CATCH_FN_PREFIX};

use syntax::codemap::{Span};
use syntax::ast::{MetaItem};
use syntax::ext::base::{Annotatable, ExtCtxt};
use syntax::print::pprust::{item_to_string};

#[derive(Debug)]
struct Params {
    code: KVSpanned<u16>,
}

fn get_error_params(ecx: &ExtCtxt, meta_item: &MetaItem) -> Params {
    // Ensure we've been supplied with a k = v meta item. Error out if not.
    let params = meta_item.expect_list(ecx, "Bad use. Expected: #[error(...)]");

    // Now grab all of the required and optional parameters.
    let req: [&'static str; 1] = ["code"];
    let kv_pairs = get_key_values(ecx, meta_item.span, &req, &[], &*params);

    // Ensure we have a code, just to keep parsing and generating errors.
    let code = kv_pairs.get("code").map_or(KVSpanned::dummy(404), |c| {
        let numeric_code = match c.node.parse() {
            Ok(n) => n,
            Err(_) => {
                let msg = "Error codes must be integer strings. (e.g. \"404\")";
                ecx.span_err(c.v_span, msg);
                404
            }
        };

        if numeric_code < 400 || numeric_code > 599 {
            ecx.span_err(c.v_span, "Error codes must be >= 400 and <= 599.");
        }

        c.clone().map(|_| { numeric_code })
    });

    Params {
        code: code,
    }
}

pub fn error_decorator(ecx: &mut ExtCtxt, sp: Span, meta_item: &MetaItem,
          annotated: &Annotatable, push: &mut FnMut(Annotatable)) {
    let parser = MetaItemParser::new(ecx, meta_item, annotated, &sp);
    let item = parser.expect_item();

    let error_params = get_error_params(ecx, meta_item);
    debug!("Error parameters are: {:?}", error_params);

    let fn_name = item.ident;
    let catch_fn_name = prepend_ident(CATCH_FN_PREFIX, &item.ident);
    let catch_code = error_params.code.node;
    let catch_fn_item = quote_item!(ecx,
         fn $catch_fn_name<'rocket>(err: ::rocket::Error,
                                    req: &'rocket ::rocket::Request<'rocket>)
                -> ::rocket::Response<'rocket> {
             // TODO: Figure out what type signature of catcher should be.
             let result = $fn_name(err, req);
             rocket::Response::with_raw_status($catch_code, result)
         }
    ).unwrap();

    debug!("{}", item_to_string(&catch_fn_item));
    push(Annotatable::Item(catch_fn_item));

    let struct_name = prepend_ident(CATCH_STRUCT_PREFIX, &item.ident);
    push(Annotatable::Item(quote_item!(ecx,
        #[allow(non_upper_case_globals)]
        pub static $struct_name: rocket::StaticCatchInfo = rocket::StaticCatchInfo {
            code: $catch_code,
            handler: $catch_fn_name
        };
    ).unwrap()));
}

