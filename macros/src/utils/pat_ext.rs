use syntax::ast::{Pat, PatKind, Ident};
use syntax::parse::token;
use syntax::codemap::DUMMY_SP;
use syntax::tokenstream::TokenTree;
use syntax::ext::quote::rt::ToTokens;
use syntax::ext::base::ExtCtxt;
use syntax::ptr::P;

pub trait PatExt {
    fn named(&self, name: &str) -> bool;
    fn ident(&self) -> Option<&Ident>;
}

impl PatExt for Pat {
    fn named(&self, name: &str) -> bool {
        match self.node {
            PatKind::Ident(_, ref ident, _) => ident.node.name.as_str() == name,
            _ => false,
        }
    }

    fn ident(&self) -> Option<&Ident> {
        match self.node {
            PatKind::Ident(_, ref ident, _) => Some(&ident.node),
            _ => None,
        }
    }
}
