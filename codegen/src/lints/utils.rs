use rustc::ty;
use rustc::hir::def_id::DefId;
use rustc::lint::LateContext;
use rustc::hir::Expr_::*;
use rustc::hir::Expr;

use syntax::symbol;

const ROCKET_TYPE: &'static [&'static str] = &["rocket", "rocket", "Rocket"];

const ROCKET_IGNITE_FN: &'static [&'static str] = &["rocket", "ignite"];
const ROCKET_IGNITE_STATIC: &'static [&'static str]
    = &["rocket", "rocket", "Rocket", "ignite"];

const ROCKET_CUSTOM_FN: &'static [&'static str] = &["rocket", "custom"];
const ROCKET_CUSTOM_STATIC: &'static [&'static str]
    = &["rocket", "rocket", "Rocket", "custom"];

const ABSOLUTE: &'static ty::item_path::RootMode = &ty::item_path::RootMode::Absolute;

/// Check if a `DefId`'s path matches the given absolute type path usage.
///
/// # Examples
/// ```rust,ignore
/// match_def_path(cx.tcx, id, &["core", "option", "Option"])
/// ```
///
/// See also the `paths` module.
pub fn match_def_path(tcx: ty::TyCtxt, def_id: DefId, path: &[&str]) -> bool {
    struct AbsolutePathBuffer {
        names: Vec<symbol::InternedString>,
    }

    impl ty::item_path::ItemPathBuffer for AbsolutePathBuffer {
        fn root_mode(&self) -> &ty::item_path::RootMode {
            ABSOLUTE
        }

        fn push(&mut self, text: &str) {
            self.names.push(symbol::Symbol::intern(text).as_str());
        }
    }

    let mut apb = AbsolutePathBuffer { names: vec![] };
    tcx.push_item_path(&mut apb, def_id);

    apb.names.len() == path.len() &&
    apb.names.iter().zip(path.iter()).all(|(a, &b)| &**a == b)
}

/// Check if the method call given in `expr` belongs to given type.
pub fn is_impl_method(cx: &LateContext, expr: &Expr, path: &[&str]) -> bool {
    let method_call = ty::MethodCall::expr(expr.id);

    let trt_id = cx.tables
        .method_map
        .get(&method_call)
        .and_then(|callee| cx.tcx.impl_of_method(callee.def_id));

    if let Some(trt_id) = trt_id {
        match_def_path(cx.tcx, trt_id, path)
    } else {
        false
    }
}

pub fn rocket_method_call<'e>(
    method: &str, cx: &LateContext, expr: &'e Expr
) -> Option<&'e [Expr]> {
    if let ExprMethodCall(ref name, _, ref exprs) = expr.node {
        if &*name.node.as_str() == method && is_impl_method(cx, expr, ROCKET_TYPE) {
            return Some(&exprs[1..]);
        }
    }

    None
}

pub fn is_rocket_start_call(cx: &LateContext, expr: &Expr) -> bool {
    if let ExprCall(ref expr, ..) = expr.node {
        if let ExprPath(ref qpath) = expr.node {
            let def_id = cx.tables.qpath_def(qpath, expr.id).def_id();
            if match_def_path(cx.tcx, def_id, ROCKET_IGNITE_FN) {
                return true
            } else if match_def_path(cx.tcx, def_id, ROCKET_IGNITE_STATIC) {
                return true
            } else if match_def_path(cx.tcx, def_id, ROCKET_CUSTOM_FN) {
                return true
            } else if is_impl_method(cx, expr, ROCKET_CUSTOM_STATIC) {
                return true
            }
        }
    }

    false
}

pub fn extract_mount_fn_def_ids(cx: &LateContext, expr: &Expr) -> Vec<DefId> {
    let mut output = Vec::new();
    // Call to into_vec
    if let ExprCall(_, ref args) = expr.node {
        if let Some(&ExprBox(ref expr)) = args.iter().next().map(|e| &e.node) {
            // Array of routes.
            if let ExprArray(ref members) = expr.node {
                for expr in members.iter() {
                    // Route::From call
                    if let ExprCall(_, ref args) = expr.node {
                        if args.len() < 1 {
                            continue;
                        }

                        // address of info struct
                        if let ExprAddrOf(_, ref expr) = args[0].node {
                            // path to info_struct
                            if let ExprPath(ref qpath) = expr.node {
                                let def = cx.tables.qpath_def(qpath, expr.id);
                                output.push(def.def_id());
                            }
                        }
                    }
                }
            }
        }
    }

    output
}
