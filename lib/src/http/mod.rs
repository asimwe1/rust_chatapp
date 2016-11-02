//! [unstable] Types that map to concepts in HTTP.
//!
//! This module exports types that map to HTTP concepts or to the underlying
//! HTTP library when needed. Because the underlying HTTP library is likely to
//! change (see <a
//! href="https://github.com/SergioBenitez/Rocket/issues/17">#17</a>), most of
//! this module should be considered unstable.
pub mod hyper;
pub mod uri;

mod cookies;
mod method;
mod content_type;

// TODO: Removed from Rocket in favor of a more flexible HTTP library.
pub use hyper::mime;

pub use self::method::Method;
pub use self::hyper::StatusCode;
pub use self::content_type::ContentType;

pub use self::cookies::{Cookie, Cookies};
