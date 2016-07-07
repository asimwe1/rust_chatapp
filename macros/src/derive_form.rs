#![allow(unused_imports)] // FIXME: Why is this coming from quote_tokens?

use syntax::ext::base::{Annotatable, ExtCtxt};
use syntax::ast::{ItemKind, Expr, MetaItem, Mutability, VariantData};
use syntax::codemap::Span;
use syntax::ext::build::AstBuilder;
use syntax::ptr::P;
use std::mem::transmute;

use syntax_ext::deriving::generic::MethodDef;
use syntax_ext::deriving::generic::{StaticStruct, Substructure, TraitDef, ty};
use syntax_ext::deriving::generic::combine_substructure as c_s;

const DEBUG: bool = false;

static ONLY_STRUCTS_ERR: &'static str = "`FromForm` can only be derived for \
    structures with named fields.";

fn get_struct_lifetime(ecx: &mut ExtCtxt, item: &Annotatable, span: Span)
        -> Option<&'static str> {
    match *item {
        Annotatable::Item(ref item) => match item.node {
            ItemKind::Struct(_, ref generics) => {
                match generics.lifetimes.len() {
                    0 => None,
                    1 => {
                        let lifetime = generics.lifetimes[0].lifetime;
                        // According to the documentation, this is safe:
                        //  Because the interner lives for the life of the
                        //  thread, this can be safely treated as an immortal
                        //  string, as long as it never crosses between threads.
                        let lifetime_name: &'static str =
                            unsafe { transmute(&*lifetime.name.as_str()) };
                        Some(lifetime_name)
                    }
                    _ => {
                        ecx.span_err(item.span, "cannot have more than one \
                            lifetime parameter when deriving `FromForm`.");
                        None
                    }
                }
            },
            _ => ecx.span_fatal(span, ONLY_STRUCTS_ERR)
        },
        _ => ecx.span_fatal(span, ONLY_STRUCTS_ERR)
    }
}

pub fn from_form_derive(ecx: &mut ExtCtxt, span: Span, meta_item: &MetaItem,
          annotated: &Annotatable, push: &mut FnMut(Annotatable)) {
    let lifetime_var = get_struct_lifetime(ecx, annotated, span);

    let trait_def = TraitDef {
        is_unsafe: false,
        span: span,
        attributes: Vec::new(),
        path: ty::Path {
            path: vec!["rocket", "form", "FromForm"],
            lifetime: lifetime_var,
            params: vec![],
            global: true,
        },
        additional_bounds: Vec::new(),
        generics: ty::LifetimeBounds::empty(),
        methods: vec![
            MethodDef {
                name: "from_form_string",
                generics: ty::LifetimeBounds::empty(),
                explicit_self: None,
                args: vec![
                    ty::Ptr(
                        Box::new(ty::Literal(ty::Path::new_local("str"))),
                        ty::Borrowed(lifetime_var, Mutability::Immutable)
                    )
                ],
                ret_ty: ty::Ty::Literal(
                    ty::Path {
                        path: vec!["std", "result", "Result"],
                        lifetime: None,
                        params: vec![
                            Box::new(ty::Ty::Self_),
                            Box::new(ty::Ty::Literal(
                                ty::Path::new(vec!["rocket", "Error"])
                            )),
                        ],
                        global: true,
                    }
                ),
                attributes: vec![],
                is_unsafe: false,
                combine_substructure: c_s(Box::new(from_form_substructure)),
                unify_fieldless_variants: false,
            }
        ],
        associated_types: vec![],
    };

    trait_def.expand(ecx, meta_item, annotated, push);
}

fn from_form_substructure(cx: &mut ExtCtxt, trait_span: Span, substr: &Substructure) -> P<Expr> {
    // Check that we specified the methods to the argument correctly.
    let arg = if substr.nonself_args.len() == 1 {
        &substr.nonself_args[0]
    } else {
        let msg = format!("incorrect number of arguments in `from_form_string`: \
            expected {}, found {}", 1, substr.nonself_args.len());
        cx.span_bug(trait_span, msg.as_str());
    };

    debug!("argument is: {:?}", arg);

    // Ensure the the fields are from a 'StaticStruct' and extract them.
    let fields = match *substr.fields {
        StaticStruct(var_data, _) => match *var_data {
            VariantData::Struct(ref fields, _) => fields,
            _ => cx.span_fatal(trait_span, ONLY_STRUCTS_ERR)
        },
        _ => cx.span_bug(trait_span, "impossible substructure in `from_form`")
    };

    // Create a vector of (ident, type) pairs, one for each field in struct.
    let mut fields_and_types = vec![];
    for field in fields {
        let ident = match field.ident {
            Some(ident) => ident,
            None => cx.span_fatal(trait_span, ONLY_STRUCTS_ERR)
        };

        fields_and_types.push((ident, &field.ty));
    }

    debug!("Fields and types: {:?}", fields_and_types);
    let mut stmts = Vec::new();

    // The thing to do when we wish to exit with an error.
    let return_err_stmt = quote_tokens!(cx,
        return Err(::rocket::Error::BadParse)
    );

    // Generating the code that checks that the number of fields is correct.
    let num_fields = fields_and_types.len();
    let initial_block = quote_block!(cx, {
        let mut items = [("", ""); $num_fields];
        let form_count = ::rocket::form::form_items($arg, &mut items);
        if form_count != items.len() {
            $return_err_stmt;
        };
    });

    stmts.extend(initial_block.unwrap().stmts);

    // Generate the let bindings for parameters that will be unwrapped and
    // placed into the final struct
    for &(ref ident, ref ty) in &fields_and_types {
        stmts.push(quote_stmt!(cx,
            let mut $ident: ::std::option::Option<$ty> = None;
        ).unwrap());
    }

    // Generating an arm for each struct field. This matches against the key and
    // tries to parse the value according to the type.
    let mut arms = vec![];
    for &(ref ident, _) in &fields_and_types {
        let ident_string = ident.to_string();
        let id_str = ident_string.as_str();
        arms.push(quote_tokens!(cx,
            $id_str => $ident = match ::rocket::form::FromFormValue::parse(v) {
                Ok(v) => Some(v),
                Err(_) => $return_err_stmt
            },
        ));
    }

    // The actual match statement. Uses the $arms generated above.
    stmts.push(quote_stmt!(cx,
       for &(k, v) in &items {
           match k {
               $arms
                _ => $return_err_stmt
           };
       }
    ).unwrap());

    // This looks complicated but just generates the boolean condition checking
    // that each parameter actually is Some(), IE, had a key/value and parsed.
    let mut failure_conditions = vec![];
    for (i, &(ref ident, _)) in (&fields_and_types).iter().enumerate() {
        if i > 0 {
            failure_conditions.push(quote_tokens!(cx, || $ident.is_none()));
        } else {
            failure_conditions.push(quote_tokens!(cx, $ident.is_none()));
        }
    }

    // The fields of the struct, which are just the let bindings declared above.
    let mut result_fields = vec![];
    for &(ref ident, _) in &fields_and_types {
        result_fields.push(quote_tokens!(cx,
            $ident: $ident.unwrap(),
        ));
    }

    // The final block: check the error conditions, and if all is well, return
    // the structure.
    let self_ident = substr.type_ident;
    let final_block = quote_block!(cx, {
        if $failure_conditions {
            $return_err_stmt;
        }

        return Ok($self_ident {
            $result_fields
        });
    });

    stmts.extend(final_block.unwrap().stmts);
    cx.expr_block(cx.block(trait_span, stmts))
}

