use std::str::FromStr;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddrV4, SocketAddrV6, SocketAddr};
use std::path::PathBuf;
use url;

use http::uri::Segments;

/// Trait to convert a dynamic path segment string to a concrete value.
///
/// This trait is used by Rocket's code generation facilities to parse dynamic
/// path segment string values into a given type. That is, when a path contains
/// a dynamic segment `<param>` where `param` has some type `T` that
/// implements `FromParam`, `T::from_param` will be called.
///
/// # Forwarding
///
/// If the conversion fails, the incoming request will be forwarded to the next
/// matching route, if any. For instance, consider the following route and
/// handler for the dynamic `"/<id>"` path:
///
/// ```rust
/// # #![feature(plugin)]
/// # #![plugin(rocket_codegen)]
/// # extern crate rocket;
/// #[get("/<id>")]
/// fn hello(id: usize) -> String {
/// # /*
///     ...
/// # */
/// # "".to_string()
/// }
/// # fn main() {  }
/// ```
///
/// If `usize::from_param` returns an `Ok(usize)` variant, the encapsulated
/// value is used as the `id` function parameter. If not, the request is
/// forwarded to the next matching route. Since there are no additional matching
/// routes, this example will result in a 404 error for requests with invalid
/// `id` values.
///
/// # Catching Errors
///
/// # `str` vs. `String`
///
/// Paths are URL encoded. As a result, the `str` `FromParam` implementation
/// returns the raw, URL encoded version of the path segment string. On the
/// other hand, `String` decodes the path parameter, but requires an allocation
/// to do so. This tradeoff is similiar to that of form values, and you should
/// use whichever makes sense for your application.
pub trait FromParam<'a>: Sized {
    /// The associated error to be returned when parsing fails.
    type Error;

    /// Parses an instance of `Self` from a dynamic path parameter string or
    /// returns an `Error` if one cannot be parsed.
    fn from_param(param: &'a str) -> Result<Self, Self::Error>;
}

impl<'a> FromParam<'a> for &'a str {
    type Error = ();
    fn from_param(param: &'a str) -> Result<&'a str, Self::Error> {
        Ok(param)
    }
}

impl<'a> FromParam<'a> for String {
    type Error = &'a str;
    fn from_param(p: &'a str) -> Result<String, Self::Error> {
        let decoder = url::percent_encoding::percent_decode(p.as_bytes());
        decoder.decode_utf8().map_err(|_| p).map(|s| s.into_owned())
    }
}

macro_rules! impl_with_fromstr {
    ($($T:ident),+) => ($(
        impl<'a> FromParam<'a> for $T {
            type Error = &'a str;
            fn from_param(param: &'a str) -> Result<Self, Self::Error> {
                $T::from_str(param).map_err(|_| param)
            }
        }
    )+)
}

impl_with_fromstr!(f32, f64, isize, i8, i16, i32, i64, usize, u8, u16, u32, u64,
       bool, IpAddr, Ipv4Addr, Ipv6Addr, SocketAddrV4, SocketAddrV6,
       SocketAddr);

/// Trait to convert _many_ dynamic path segment strings to a concrete value.
///
/// This is the `..` analog to [FromParam](trait.FromParam.html), and its
/// functionality is identical to it with one exception: this trait applies to
/// segment parameters of the form `<param..>`, where `param` is of some type
/// `T` that implements `FromSegments`. `T::from_segments` is called to convert
/// the matched segments (via the
/// [Segments](/rocket/http/uri/struct.Segments.html) iterator) into the
/// implementing type.
pub trait FromSegments<'a>: Sized {
    /// The associated error to be returned when parsing fails.
    type Error;

    /// Parses an instance of `Self` from many dynamic path parameter strings or
    /// returns an `Error` if one cannot be parsed.
    fn from_segments(segments: Segments<'a>) -> Result<Self, Self::Error>;
}

impl<'a> FromSegments<'a> for Segments<'a> {
    type Error = ();
    fn from_segments(segments: Segments<'a>) -> Result<Segments<'a>, ()> {
        Ok(segments)
    }
}

impl<'a> FromSegments<'a> for PathBuf {
    type Error = ();
    fn from_segments(segments: Segments<'a>) -> Result<PathBuf, ()> {
        Ok(segments.collect())
    }
}
