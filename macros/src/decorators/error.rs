use utils::*;
use ::{CATCH_STRUCT_PREFIX, CATCH_FN_PREFIX};

use syntax::codemap::{Span};
use syntax::ast::{MetaItem};
use syntax::ext::base::{Annotatable, ExtCtxt};
use parser::ErrorParams;

pub fn error_decorator(ecx: &mut ExtCtxt, sp: Span, meta_item: &MetaItem,
          annotated: &Annotatable, push: &mut FnMut(Annotatable)) {
    let error = ErrorParams::from(ecx, sp, meta_item, annotated);

    let user_fn_name = error.annotated_fn.ident();
    let catch_fn_name = user_fn_name.prepend(CATCH_FN_PREFIX);
    let code = error.code.node;
    emit_item(push, quote_item!(ecx,
         fn $catch_fn_name<'rocket>(err: ::rocket::Error,
                                    req: &'rocket ::rocket::Request<'rocket>)
                                    -> ::rocket::Response<'rocket> {
             rocket::Response::with_raw_status($code, $user_fn_name(err, req))
         }
    ).expect("catch function"));

    let struct_name = user_fn_name.prepend(CATCH_STRUCT_PREFIX);
    emit_item(push, quote_item!(ecx,
        #[allow(non_upper_case_globals)]
        pub static $struct_name: rocket::StaticCatchInfo = rocket::StaticCatchInfo {
            code: $code,
            handler: $catch_fn_name
        };
    ).expect("catch info struct"));
}
