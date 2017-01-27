use http::Header;

pub use cookie::Cookie;
pub use cookie::CookieJar;
pub use cookie::CookieBuilder;

/// Type alias to a `'static` CookieJar.
///
/// A `CookieJar` should never be used without a `'static` lifetime. As a
/// result, you should always use this alias.
pub type Cookies = self::CookieJar<'static>;

impl<'c> From<Cookie<'c>> for Header<'static> {
    fn from(cookie: Cookie) -> Header<'static> {
        Header::new("Set-Cookie", cookie.encoded().to_string())
    }
}
