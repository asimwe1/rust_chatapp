extern crate syntax_pos;

mod utils;

use self::utils::*;

use ::{ROUTE_ATTR, ROUTE_INFO_ATTR};

use std::mem::transmute;
use std::collections::{HashSet, HashMap};

use rustc::lint::Level;
use rustc::lint::{LateContext, LintContext, LintPass, LateLintPass, LintArray};
use rustc::hir::{Item, Expr, Crate};
use rustc::hir::def::Def;
use rustc::hir::def_id::DefId;
use rustc::ty::Ty;
use rustc::hir::intravisit::{FnKind};
use rustc::hir::{FnDecl, Body};
use rustc::hir::Ty_::TyPath;
use self::syntax_pos::Span;

use syntax::symbol::Symbol as Name;
use syntax::ast::NodeId;

const STATE_TYPE: &'static [&'static str] = &["rocket", "request", "state", "State"];

#[derive(Debug, Default)]
pub struct ManagedStateLint {
    // All of the types that were requested as managed state.
    // (fn_name, fn_span, info_struct_def_id, req_type, req_param_span)
    requested: Vec<(Name, Span, DefId, Ty<'static>, Span)>,
    // The DefId of all of the route infos for the mounted routes.
    mounted: HashSet<DefId>,
    // The names of all of the routes that were declared.
    info_structs: HashMap<Name, DefId>,
    // The name, span, and info DefId for all declared route functions.
    declared: Vec<(Name, Span, DefId)>,
    // The expressions that were passed into a `.manage` call.
    managed: Vec<(Ty<'static>, Span)>,
    // Span for rocket::ignite() or rocket::custom().
    start_call: Vec<Span>,
}

declare_lint!(UNMOUNTED_ROUTE, Warn, "Warn on routes that are unmounted.");
declare_lint!(UNMANAGED_STATE, Warn, "Warn on declared use on unmanaged state.");

impl<'tcx> LintPass for ManagedStateLint {
    fn get_lints(&self) -> LintArray {
        lint_array!(UNMANAGED_STATE, UNMOUNTED_ROUTE)
    }
}

impl<'a, 'tcx> LateLintPass<'a, 'tcx> for ManagedStateLint {
    fn check_expr(&mut self, cx: &LateContext<'a, 'tcx>, expr: &'tcx Expr) {
        if let Some(args) = rocket_method_call("manage", cx, expr) {
            let expr = &args[0];
            if let Some(ty) = cx.tables.expr_ty_opt(expr) {
                let casted = unsafe { transmute(ty) };
                self.managed.push((casted, expr.span));
            }
        }

        if let Some(args) = rocket_method_call("mount", cx, expr) {
            for def_id in extract_mount_fn_def_ids(cx, &args[1]) {
                self.mounted.insert(def_id);
            }
        }

        if is_rocket_start_call(cx, expr) {
            self.start_call.push(expr.span);
        }
    }

    fn check_item(&mut self, cx: &LateContext<'a, 'tcx>, item: &'tcx Item) {
        // Return early if this is not a route info structure.
        if !item.attrs.iter().any(|attr| attr.check_name(ROUTE_INFO_ATTR)) {
            return;
        }

        if let Some(def_id) = cx.tcx.map.opt_local_def_id(item.id) {
            self.info_structs.insert(item.name, def_id);
        }
    }

    fn check_fn(&mut self,
                cx: &LateContext<'a, 'tcx>,
                kind: FnKind<'tcx>,
                decl: &'tcx FnDecl,
                _: &'tcx Body,
                fn_sp: Span,
                fn_id: NodeId)
    {
        // Get the name of the function, if any.
        let fn_name = match kind {
            FnKind::ItemFn(name, ..) => name,
            _ => return
        };

        // Figure out if this is a route function by trying to find the
        // `ROUTE_ATTR` attribute and extracing the info struct's name from it.
        let attr_value = kind.attrs().iter().filter_map(|attr| {
            if !attr.check_name(ROUTE_ATTR) {
                None
            } else {
                attr.value.meta_item_list().and_then(|list| list[0].name())
            }
        }).next();

        // Try to get the DEF_ID using the info struct's name. Return early if
        // anything goes awry.
        let def_id = match attr_value {
            Some(val) if self.info_structs.contains_key(&val) => {
                self.info_structs.get(&val).unwrap()
            }
            _ => return
        };

        // Add this to the list of declared routes to check for mounting later
        // unless unmounted routes were explicitly allowed for this function.
        if cx.current_level(UNMOUNTED_ROUTE) != Level::Allow {
            self.declared.push((fn_name, fn_sp, def_id.clone()));
        }

        // If unmanaged state was explicitly allowed for this function, don't
        // record any additional information. Just return now.
        if cx.current_level(UNMANAGED_STATE) == Level::Allow {
            return;
        }

        // Collect all of the `State` types into `tys`.
        let mut tys: Vec<Ty<'static>> = vec![];
        if let Some(sig) = cx.tables.liberated_fn_sigs.get(&fn_id) {
            for input_ty in sig.inputs() {
                let def_id = match input_ty.ty_to_def_id() {
                    Some(id) => id,
                    None => continue
                };

                if !match_def_path(cx.tcx, def_id, STATE_TYPE) {
                    continue;
                }

                if let Some(inner_type) = input_ty.walk_shallow().next() {
                    let casted = unsafe { transmute(inner_type) };
                    tys.push(casted);
                }
            }
        }

        // Collect all of the spans for the `State` parameters.
        let mut spans = vec![];
        for input in decl.inputs.iter() {
            let id = input.id;
            if let TyPath(ref qpath) = input.node {
                if let Def::Struct(defid) = cx.tables.qpath_def(qpath, id) {
                    if match_def_path(cx.tcx, defid, STATE_TYPE) {
                        spans.push(input.span);
                    }
                }
            }
        }

        // Sanity check: we should have as many spans as types.
        if tys.len() != spans.len() {
            panic!("Internal lint error: mismatched type/spans: {}/{}",
                   tys.len(), spans.len());
        }

        // Insert the information we've collected.
        for (ty, span) in tys.into_iter().zip(spans.into_iter()) {
            self.requested.push((fn_name, fn_sp, def_id.clone(), ty, span));
        }
    }

    fn check_crate_post(&mut self, cx: &LateContext<'a, 'tcx>, _: &'tcx Crate) {
        // Record the start function span if we found one, and only one.
        // TODO: Try to find the _right_ one using some heuristics.
        let start_call_sp = match self.start_call.len() == 1 {
            true => Some(self.start_call[0]),
            false => None
        };

        // Emit a warning for all unmounted, declared routes.
        for &(route_name, fn_sp, info_def_id) in self.declared.iter() {
            if !self.mounted.contains(&info_def_id) {
                let msg = format!("the '{}' route is not mounted", route_name);
                let mut b = cx.struct_span_lint(UNMOUNTED_ROUTE, fn_sp, &msg);
                b.note("Rocket will not dispatch requests to unmounted routes");
                if let Some(start_sp) = start_call_sp {
                    b.span_help(start_sp, "maybe missing a call to 'mount' here?");
                }

                b.emit();
            }
        }

        let managed_types: HashSet<Ty> = self.managed.iter()
            .map(|&(ty, _)| ty)
            .collect();

        for &(_, _, info_def_id, ty, sp) in self.requested.iter() {
            // Don't warn on unmounted routes.
            if !self.mounted.contains(&info_def_id) {
                continue
            }

            if !managed_types.contains(&ty) {
                let m = format!("'{}' is not currently being managed by Rocket", ty);
                let mut b = cx.struct_span_lint(UNMANAGED_STATE, sp, &m);
                b.note("this 'State' request guard will always fail");
                if let Some(start_sp) = start_call_sp {
                    let msg = format!("maybe missing a call to 'manage' here?");
                    b.span_help(start_sp, &msg);
                }

                b.emit()
            }
        }
    }
}
