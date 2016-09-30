//! Types and traits that deal with request parsing and handling.

mod request;
mod param;
mod from_request;

pub use self::request::Request;
pub use self::from_request::FromRequest;
pub use self::param::{FromParam, FromSegments};
pub use hyper::header::CookiePair as Cookie;

// Unexported Hyper types.
#[doc(hidden)] pub use hyper::server::Request as HyperRequest;
#[doc(hidden)] pub use hyper::header::Headers as HyperHeaders;
#[doc(hidden)] pub use hyper::header::Cookie as HyperCookie;

use hyper::header::CookieJar;
pub type Cookies = CookieJar<'static>;
