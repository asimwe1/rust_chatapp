use devise::{*, ext::{TypeExt, SpanDiagnosticExt}};

use crate::exports::*;
use crate::proc_macro2::Span;
use crate::syn_ext::NameSource;

pub struct FormField {
    pub span: Span,
    pub name: NameSource,
}

#[derive(FromMeta)]
pub struct FieldAttr {
    pub name: Option<FormField>,
    pub validate: Option<syn::Expr>,
}

impl FieldAttr {
    const NAME: &'static str = "field";
}

pub(crate) trait FieldExt {
    fn ident(&self) -> &syn::Ident;
    fn field_name(&self) -> Result<String>;
    fn stripped_ty(&self) -> syn::Type;
    fn name_view(&self) -> Result<syn::Expr>;
}

impl FromMeta for FormField {
    fn from_meta(meta: &MetaItem) -> Result<Self> {
        // These are used during parsing.
        const CONTROL_CHARS: &[char] = &['&', '=', '?', '.', '[', ']'];

        fn is_valid_field_name(s: &str) -> bool {
            // The HTML5 spec (4.10.18.1) says 'isindex' is not allowed.
            if s == "isindex" || s.is_empty() {
                return false
            }

            // We allow all visible ASCII characters except `CONTROL_CHARS`.
            s.chars().all(|c| c.is_ascii_graphic() && !CONTROL_CHARS.contains(&c))
        }

        let name = NameSource::from_meta(meta)?;
        if !is_valid_field_name(name.name()) {
            let chars = CONTROL_CHARS.iter()
                .map(|c| format!("{:?}", c))
                .collect::<Vec<_>>()
                .join(", ");

            return Err(meta.value_span()
                .error("invalid form field name")
                .help(format!("field name cannot be `isindex` or contain {}", chars)));
        }

        Ok(FormField { span: meta.value_span(), name })
    }
}

impl FieldExt for Field<'_> {
    fn ident(&self) -> &syn::Ident {
        self.ident.as_ref().expect("named")
    }

    fn field_name(&self) -> Result<String> {
        let mut fields = FieldAttr::from_attrs(FieldAttr::NAME, &self.attrs)?
            .into_iter()
            .filter_map(|attr| attr.name);

        let name = fields.next()
            .map(|f| f.name)
            .unwrap_or_else(|| NameSource::from(self.ident().clone()));

        if let Some(field) = fields.next() {
            return Err(field.span
                .error("duplicate form field renaming")
                .help("a field can only be renamed once"));
        }

        Ok(name.to_string())
    }

    fn stripped_ty(&self) -> syn::Type {
        self.ty.with_stripped_lifetimes()
    }

    fn name_view(&self) -> Result<syn::Expr> {
        let field_name = self.field_name()?;
        Ok(syn::parse_quote!(#_form::NameBuf::from((__c.__parent, #field_name))))
    }
}

pub fn validators<'v>(
    field: Field<'v>,
    out: &'v syn::Ident,
    local: bool,
) -> Result<impl Iterator<Item = syn::Expr> + 'v> {
    use syn::visit_mut::VisitMut;

    struct ValidationMutator<'a> {
        field: syn::Expr,
        self_expr: syn::Expr,
        form: &'a syn::Ident,
        visited: bool,
        rec: bool,
    }

    impl<'a> ValidationMutator<'a> {
        fn new(field: &'a syn::Ident, form: &'a syn::Ident) -> Self {
            let self_expr = syn::parse_quote!(&#form.#field);
            let field = syn::parse_quote!(&#field);
            ValidationMutator { field, self_expr, form, visited: false, rec: false }
        }
    }

    impl VisitMut for ValidationMutator<'_> {
        fn visit_expr_call_mut(&mut self, call: &mut syn::ExprCall) {
            syn::visit_mut::visit_expr_call_mut(self, call);

            let ident = if self.rec { &self.self_expr } else { &self.field };
            if !self.visited {
                call.args.insert(0, ident.clone());
                self.visited = true;
            }
        }

        fn visit_ident_mut(&mut self, i: &mut syn::Ident) {
            if i == "self" {
                *i = self.form.clone();
                self.rec = true;
            }

            syn::visit_mut::visit_ident_mut(self, i);
        }
    }

    Ok(FieldAttr::from_attrs(FieldAttr::NAME, &field.attrs)?
        .into_iter()
        .filter_map(|a| a.validate)
        .map(move |mut expr| {
            let mut mutator = ValidationMutator::new(field.ident(), out);
            mutator.visit_expr_mut(&mut expr);
            (expr, mutator.rec)
        })
        .filter(move |(_, global)| local != *global)
        .map(|(e, _)| e))
}
