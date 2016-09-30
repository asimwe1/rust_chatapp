mod request;
mod from_request;

pub use self::request::Request;
pub use self::from_request::FromRequest;

#[doc(hidden)]
pub use hyper::server::Request as HyperRequest;
#[doc(hidden)]
pub use hyper::header::Headers as HyperHeaders;
#[doc(hidden)]
pub use hyper::header::Cookie as HyperCookie;

pub use hyper::header::CookiePair as Cookie;

use hyper::header::CookieJar;
pub type Cookies = CookieJar<'static>;
