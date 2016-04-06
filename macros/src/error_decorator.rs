use utils::*;
use super::{CATCH_STRUCT_PREFIX, CATCH_FN_PREFIX};

use route_decorator::get_fn_decl;

use syntax::codemap::{Span};
use syntax::ast::{MetaItem};
use syntax::ext::base::{Annotatable, ExtCtxt};
use syntax::print::pprust::{item_to_string};

#[allow(dead_code)]
const DEBUG: bool = true;

#[derive(Debug)]
struct Params {
    code: KVSpanned<u16>,
}

fn get_error_params(ecx: &mut ExtCtxt, meta_item: &MetaItem) -> Params {
    assert_meta_item_list(ecx, meta_item, "error");

    // Ensure we can unwrap the k = v params.
    let params = meta_item.node.get_list_items().unwrap();

    // Now grab all of the required and optional parameters.
    let req: [&'static str; 1] = ["code"];
    let kv_pairs = get_key_values(ecx, meta_item.span, &req, &[], &*params);

    // Ensure we have a code, just to keep parsing and generating errors.
    let code = kv_pairs.get("code").map_or(dummy_kvspan(404), |c| {
        let numeric_code = match c.node.parse() {
            Ok(n) => n,
            Err(_) => {
                let msg = "Error codes must be integer strings. (e.g. \"404\")";
                ecx.span_err(c.v_span, msg);
                404
            }
        };

        if numeric_code < 100 || numeric_code > 599 {
            ecx.span_err(c.v_span, "Error codes must be >= 100 and <= 599.");
        }

        c.clone().map(|_| { numeric_code })
    });

    Params {
        code: code,
    }
}

pub fn error_decorator(ecx: &mut ExtCtxt, sp: Span, meta_item: &MetaItem,
          annotated: &Annotatable, push: &mut FnMut(Annotatable)) {
    let (item, _fn_decl) = get_fn_decl(ecx, sp, annotated);
    let error_params = get_error_params(ecx, meta_item);
    debug!("Error parameters are: {:?}", error_params);

    let fn_name = item.ident;
    let catch_fn_name = prepend_ident(CATCH_FN_PREFIX, &item.ident);
    let catch_fn_item = quote_item!(ecx,
         fn $catch_fn_name<'rocket>(_req: rocket::Request<'rocket>)
                -> rocket::Response<'rocket> {
             // TODO: Figure out what type signature of catcher should be.
             let result = $fn_name();
             rocket::Response::new(result)
         }
    ).unwrap();

    debug!("{}", item_to_string(&catch_fn_item));
    push(Annotatable::Item(catch_fn_item));

    let struct_name = prepend_ident(CATCH_STRUCT_PREFIX, &item.ident);
    let catch_code = error_params.code.node;
    push(Annotatable::Item(quote_item!(ecx,
        #[allow(non_upper_case_globals)]
        pub static $struct_name: rocket::StaticCatchInfo = rocket::StaticCatchInfo {
            code: $catch_code,
            handler: $catch_fn_name
        };
    ).unwrap()));
}

