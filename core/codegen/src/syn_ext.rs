//! Extensions to `syn` types.

use crate::syn::{self, Ident, ext::IdentExt as _};
use crate::proc_macro2::Span;

pub trait IdentExt {
    fn prepend(&self, string: &str) -> syn::Ident;
    fn append(&self, string: &str) -> syn::Ident;
    fn with_span(self, span: Span) -> syn::Ident;
    fn rocketized(&self) -> syn::Ident;
}

pub trait ReturnTypeExt {
    fn ty(&self) -> Option<&syn::Type>;
}

pub trait TokenStreamExt {
    fn respanned(&self, span: crate::proc_macro2::Span) -> Self;
}

pub trait FnArgExt {
    fn typed(&self) -> Option<(&syn::Ident, &syn::Type)>;
    fn wild(&self) -> Option<&syn::PatWild>;
}

impl IdentExt for syn::Ident {
    fn prepend(&self, string: &str) -> syn::Ident {
        syn::Ident::new(&format!("{}{}", string, self.unraw()), self.span())
    }

    fn append(&self, string: &str) -> syn::Ident {
        syn::Ident::new(&format!("{}{}", self, string), self.span())
    }

    fn with_span(mut self, span: Span) -> syn::Ident {
        self.set_span(span);
        self
    }

    fn rocketized(&self) -> syn::Ident {
        self.prepend(crate::ROCKET_IDENT_PREFIX)
    }
}

impl ReturnTypeExt for syn::ReturnType {
    fn ty(&self) -> Option<&syn::Type> {
        match self {
            syn::ReturnType::Default => None,
            syn::ReturnType::Type(_, ty) => Some(ty),
        }
    }
}

impl TokenStreamExt for crate::proc_macro2::TokenStream {
    fn respanned(&self, span: crate::proc_macro2::Span) -> Self {
        self.clone().into_iter().map(|mut token| {
            token.set_span(span);
            token
        }).collect()
    }
}

impl FnArgExt for syn::FnArg {
    fn typed(&self) -> Option<(&Ident, &syn::Type)> {
        match self {
            syn::FnArg::Typed(arg) => match *arg.pat {
                syn::Pat::Ident(ref pat) => Some((&pat.ident, &arg.ty)),
                _ => None
            }
            _ => None,
        }
    }

    fn wild(&self) -> Option<&syn::PatWild> {
        match self {
            syn::FnArg::Typed(arg) => match *arg.pat {
                syn::Pat::Wild(ref pat) => Some(pat),
                _ => None
            }
            _ => None,
        }
    }
}
