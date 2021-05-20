/// The literal `*` URI.
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub struct Asterisk;

impl std::fmt::Display for Asterisk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        "*".fmt(f)
    }
}
