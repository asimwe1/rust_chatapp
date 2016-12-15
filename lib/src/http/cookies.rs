use http;

pub use http::hyper::header::CookiePair as Cookie;

pub type Cookies = http::hyper::header::CookieJar<'static>;
