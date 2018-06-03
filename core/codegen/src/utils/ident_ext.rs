use std::fmt::Display;
use syntax::ast::Ident;
use syntax::symbol::Symbol;

pub trait IdentExt {
    fn prepend<T: Display>(&self, other: T) -> Ident;
    fn append<T: Display>(&self, other: T) -> Ident;
}

impl IdentExt for Ident {
    fn prepend<T: Display>(&self, other: T) -> Ident {
        let new_ident = format!("{}{}", other, self.name);
        Ident::new(Symbol::intern(&new_ident), self.span)
    }

    fn append<T: Display>(&self, other: T) -> Ident {
        let new_ident = format!("{}{}", self.name, other);
        Ident::new(Symbol::intern(&new_ident), self.span)
    }
}
