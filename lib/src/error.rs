/// [unstable] Error type for Rocket. Likely to change.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Error {
    /// The request method was bad.
    BadMethod,
    /// The value could not be parsed.
    BadParse,
    /// There was no such route.
    NoRoute, // TODO: Add a chain of routes attempted.
    /// The error was internal.
    Internal,
    /// The requested key/index does not exist.
    NoKey,
}
