use std::fmt::Display;
use syntax::parse::token::str_to_ident;
use syntax::ast::Ident;

pub trait IdentExt {
    fn prepend<T: Display>(&self, other: T) -> Ident;
    fn append<T: Display>(&self, other: T) -> Ident;
}

impl IdentExt for Ident {
    fn prepend<T: Display>(&self, other: T) -> Ident {
        let new_ident = format!("{}{}", other, self.name);
        str_to_ident(new_ident.as_str())
    }

    fn append<T: Display>(&self, other: T) -> Ident {
        let new_ident = format!("{}{}", self.name, other);
        str_to_ident(new_ident.as_str())
    }
}
