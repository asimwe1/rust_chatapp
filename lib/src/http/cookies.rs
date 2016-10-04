pub use http::hyper::HyperCookiePair as Cookie;

use http;
pub type Cookies = http::hyper::HyperCookieJar<'static>;
