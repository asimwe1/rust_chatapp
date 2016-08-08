mod request;
mod from_request;

pub use self::request::Request;
pub use self::from_request::FromRequest;

pub use hyper::server::Request as HyperRequest;
pub use hyper::header::Headers as HyperHeaders;
pub use hyper::header::Cookie as HyperCookie;
use hyper::header::CookieJar;

pub type Cookies = CookieJar<'static>;
