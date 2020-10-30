//! Types and traits for request parsing and handling.

mod request;
mod from_param;
mod from_request;

#[cfg(test)]
mod tests;

pub use self::request::Request;
pub use self::from_request::{FromRequest, Outcome};
pub use self::from_param::{FromParam, FromSegments};

#[doc(inline)]
pub use crate::response::flash::FlashMessage;

/// Store and immediately retrieve a value `$v` in `$request`'s local cache
/// using a locally generated anonymous type to avoid type conflicts.
///
/// # Example
///
/// ```rust
/// use rocket::request;
///
/// # rocket::Request::example(rocket::http::Method::Get, "/uri", |request| {
/// // The first store into local cache for a given type wins.
/// let value = request.local_cache(|| "hello");
/// assert_eq!(*request.local_cache(|| "hello"), "hello");
///
/// // The following return the cached, previously stored value for the type.
/// assert_eq!(*request.local_cache(|| "goodbye"), "hello");
///
/// // We cannot cache different values of the same type; we _must_ use a proxy
/// // type. To avoid the need to write these manually, use `local_cache!`,
/// // which generates one of the fly.
/// assert_eq!(*request::local_cache!(request, "hello"), "hello");
/// assert_eq!(*request::local_cache!(request, "goodbye"), "goodbye");
/// # });
/// ```
#[macro_export]
macro_rules! local_cache {
    ($request:expr, $v:expr) => ({
        struct Local<T>(T);
        &$request.local_cache(move || Local($v)).0
    })
}

#[doc(inline)]
pub use local_cache;
