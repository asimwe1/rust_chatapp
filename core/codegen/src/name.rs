use crate::syn::{self, Ident, ext::IdentExt};
use crate::proc_macro2::Span;

/// A "name" read by codegen, which may or may not be a valid identifier. A
/// `Name` is typically constructed indirectly via FromMeta, or From<Ident> or
/// directly from a string via `Name::new()`.
///
/// Some "names" in Rocket include:
///   * Dynamic parameter: `name` in `<name>`
///   * Renamed fields: `foo` in #[field(name = "foo")].
///
/// `Name` implements Hash, PartialEq, and Eq, and additionally PartialEq<S> for
/// all types `S: AsStr<str>`. These implementations all compare the value of
/// `name()` only.
#[derive(Debug, Clone)]
pub struct Name {
    value: String,
    span: Span,
}

impl Name {
    /// Creates a new `Name` from the string `name` and span `span`. If
    /// `name` is a valid ident, the ident is stored as well.
    pub fn new<S: Into<String>>(name: S, span: Span) -> Self {
        Name { value: name.into(), span }
    }

    /// Returns the name as a string. Notably, if `self` was constructed from an
    /// Ident this method returns a name *without* an `r#` prefix.
    pub fn as_str(&self) -> &str {
        &self.value
    }

    pub fn span(&self) -> Span {
        self.span
    }
}

impl devise::FromMeta for Name {
    fn from_meta(meta: &devise::MetaItem) -> devise::Result<Self> {
        use devise::ext::SpanDiagnosticExt;

        if let syn::Lit::Str(s) = meta.lit()? {
            return Ok(Name::new(s.value(), s.span()));
        }

        Err(meta.value_span().error("invalid value: expected string literal"))
    }
}

impl quote::ToTokens for Name {
    fn to_tokens(&self, tokens: &mut devise::proc_macro2::TokenStream) {
        self.as_str().to_tokens(tokens)
    }
}

impl From<&Ident> for Name {
    fn from(ident: &Ident) -> Self {
        Name::new(ident.unraw().to_string(), ident.span())
    }
}

impl From<Ident> for Name {
    fn from(ident: Ident) -> Self {
        Name::new(ident.unraw().to_string(), ident.span())
    }
}

impl AsRef<str> for Name {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::hash::Hash for Name {
    fn hash<H: std::hash::Hasher>(&self, hasher: &mut H) {
        self.as_str().hash(hasher)
    }
}

impl std::ops::Deref for Name {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl Eq for Name { }

impl<S: AsRef<str> + ?Sized> PartialEq<S> for Name {
    fn eq(&self, other: &S) -> bool {
        self.as_str() == other.as_ref()
    }
}

impl std::fmt::Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_str().fmt(f)
    }
}
