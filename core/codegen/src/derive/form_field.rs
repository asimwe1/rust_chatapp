use devise::{*, ext::{TypeExt, SpanDiagnosticExt}};

use syn::visit_mut::VisitMut;
use syn::visit::Visit;

use crate::exports::*;
use crate::proc_macro2::{Span, TokenStream, TokenTree};
use crate::name::Name;

pub struct FormField {
    pub span: Span,
    pub name: Name,
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

        let name = Name::from_meta(meta)?;
        if !is_valid_field_name(name.as_str()) {
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
            .unwrap_or_else(|| Name::from(self.ident().clone()));

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
        let name_view = quote_spanned! { self.span() =>
            #_form::NameBuf::from((__c.__parent, #field_name))
        };

        Ok(syn::parse2(name_view).unwrap())
    }
}

struct RecordMemberAccesses(Vec<syn::Member>);

impl<'a> Visit<'a> for RecordMemberAccesses {
    fn visit_expr_field(&mut self, i: &syn::ExprField) {
        if let syn::Expr::Path(e) = &*i.base {
            if e.path.is_ident("self") {
                self.0.push(i.member.clone());
            }
        }

        syn::visit::visit_expr_field(self, i);
    }
}

struct ValidationMutator<'a> {
    field: &'a syn::Ident,
    parent: &'a syn::Ident,
    local: bool,
    visited: bool,
}

impl ValidationMutator<'_> {
    fn visit_token_stream(&mut self, tt: TokenStream) -> TokenStream {
        use quote::{ToTokens, TokenStreamExt};
        use TokenTree::*;

        let mut iter = tt.into_iter();
        let mut stream = TokenStream::new();
        while let Some(tt) = iter.next() {
            match tt {
                Ident(s3lf) if s3lf == "self" => {
                    match (iter.next(), iter.next()) {
                        (Some(Punct(p)), Some(Ident(i))) if p.as_char() == '.' => {
                            let field = syn::parse_quote!(#s3lf #p #i);
                            let mut expr = syn::Expr::Field(field);
                            self.visit_expr_mut(&mut expr);
                            expr.to_tokens(&mut stream);
                        },
                        (tt1, tt2) => stream.append_all(&[Some(Ident(s3lf)), tt1, tt2]),
                    }
                },
                TokenTree::Group(group) => {
                    let tt = self.visit_token_stream(group.stream());
                    let mut new = proc_macro2::Group::new(group.delimiter(), tt);
                    new.set_span(group.span());
                    let group = TokenTree::Group(new);
                    stream.append(group);
                }
                tt => stream.append(tt),
            }
        }

        stream
    }
}

impl VisitMut for ValidationMutator<'_> {
    fn visit_expr_call_mut(&mut self, call: &mut syn::ExprCall) {
        syn::visit_mut::visit_expr_call_mut(self, call);

        // Only modify the first call we see.
        if self.visited { return; }

        let (parent, field) = (self.parent, self.field);
        let form_field = match self.local {
            true => syn::parse2(quote_spanned!(field.span() => &#field)).unwrap(),
            false => syn::parse2(quote_spanned!(field.span() => &#parent.#field)).unwrap(),
        };

        call.args.insert(0, form_field);
        self.visited = true;
    }

    fn visit_ident_mut(&mut self, i: &mut syn::Ident) {
        if !self.local && i == "self" {
            *i = self.parent.clone();
        }
    }

    fn visit_macro_mut(&mut self, mac: &mut syn::Macro) {
        mac.tokens = self.visit_token_stream(mac.tokens.clone());
        syn::visit_mut::visit_macro_mut(self, mac);
    }

    fn visit_expr_mut(&mut self, i: &mut syn::Expr) {
        // If this is a local, replace accesses of `self.field` with `field`.
        if let syn::Expr::Field(e) = i {
            if let syn::Expr::Path(e) = &*e.base {
                if e.path.is_ident("self") && self.local {
                    let new_expr = &self.field;
                    *i = syn::parse_quote!(#new_expr);
                }
            }
        }

        return syn::visit_mut::visit_expr_mut(self, i);
    }
}

pub fn validators<'v>(
    field: Field<'v>,
    parent: &'v syn::Ident, // field ident (if local) or form ident (if !local)
    local: bool,
) -> Result<impl Iterator<Item = syn::Expr> + 'v> {
    Ok(FieldAttr::from_attrs(FieldAttr::NAME, &field.attrs)?
        .into_iter()
        .filter_map(|a| a.validate)
        .map(move |expr| {
            let mut members = RecordMemberAccesses(vec![]);
            members.visit_expr(&expr);

            let field_ident = field.ident();
            let is_local_validation = members.0.iter()
                .all(|member| match member {
                    syn::Member::Named(i) => i == field_ident,
                    _ => false
                });

            (expr, is_local_validation)
        })
        .filter(move |(_, is_local)| *is_local == local)
        .map(move |(mut expr, _)| {
            let field = field.ident();
            let mut v = ValidationMutator { parent, local, field, visited: false };
            v.visit_expr_mut(&mut expr);
            expr
        }))
}
