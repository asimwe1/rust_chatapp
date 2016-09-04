use syntax::ast::{LitKind, NestedMetaItem, MetaItemKind, Lit};
use syntax::parse::token::InternedString;

pub trait MetaItemExt {
    fn name_value(&self) -> Option<(&InternedString, &Lit)>;
    fn str_lit(&self) -> Option<&InternedString>;
    fn int_lit(&self) -> Option<u64>;
}

impl MetaItemExt for NestedMetaItem {
    fn name_value(&self) -> Option<(&InternedString, &Lit)> {
        self.meta_item().and_then(|mi| match mi.node {
            MetaItemKind::NameValue(ref s, ref l) => Some((s, l)),
            _ => None,
        })
    }

    fn str_lit(&self) -> Option<&InternedString> {
        self.literal().and_then(|lit| match lit.node {
            LitKind::Str(ref s, _) => Some(s),
            _ => None,
        })
    }

    fn int_lit(&self) -> Option<u64> {
        self.literal().and_then(|lit| match lit.node {
            LitKind::Int(n, _) => Some(n),
            _ => None,
        })
    }
}
