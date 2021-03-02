use std::hash::{Hash, Hasher};

use devise::{syn, FromMeta, MetaItem, Result};

use crate::name::Name;
use crate::proc_macro2::Span;
use crate::proc_macro_ext::StringLit;
use crate::http::uri;


impl Dynamic {
    pub fn is_wild(&self) -> bool {
        self.value == "_"
    }
}

impl FromMeta for Dynamic {
    fn from_meta(meta: &MetaItem) -> Result<Self> {
        let string = StringLit::from_meta(meta)?;
        let span = string.subspan(1..string.len() + 1);

        // We don't allow `_`. We abuse `uri::Query` to enforce this.
        Ok(Dynamic::parse::<uri::Query>(&string, span)?)
    }
}

impl PartialEq for Dynamic {
    fn eq(&self, other: &Dynamic) -> bool {
        self.value == other.value
    }
}

impl Eq for Dynamic {}

impl Hash for Dynamic {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state)
    }
}
