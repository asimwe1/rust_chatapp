#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Error {
    BadMethod,
    BadParse,
    NoRoute, // FIXME: Add a chain of routes attempted.
    Internal,
    NoKey
}
