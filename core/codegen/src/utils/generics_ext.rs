use syntax::ast::{GenericParam, GenericParamKind};

pub trait GenericParamExt {
    /// Returns `true` if `self` is of kind `lifetime`.
    fn is_lifetime(&self) -> bool;
}

impl GenericParamExt for GenericParam {
    fn is_lifetime(&self) -> bool {
        match self.kind {
            GenericParamKind::Lifetime => true,
            _ => false
        }
    }
}
