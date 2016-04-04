#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Error {
    BadMethod,
    BadParse,
    NoRoute,
    NoKey
}
