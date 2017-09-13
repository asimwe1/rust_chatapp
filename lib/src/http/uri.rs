//! Borrowed and owned string types for absolute URIs.

use std::fmt;
use std::convert::From;
use std::borrow::Cow;
use std::str::Utf8Error;
use std::sync::atomic::{AtomicIsize, Ordering};

use http::RawStr;

use url;

/// Index (start, end) into a string, to prevent borrowing.
type Index = (usize, usize);

/// Representation of an empty segment count.
const EMPTY: isize = -1;

// TODO: Reconsider deriving PartialEq and Eq to make "//a/b" == "/a/b".
/// Borrowed string type for absolute URIs.
#[derive(Debug)]
pub struct URI<'a> {
    uri: Cow<'a, str>,
    path: Index,
    query: Option<Index>,
    fragment: Option<Index>,
    // The cached segment count. `EMPTY` is used to represent no segment count.
    segment_count: AtomicIsize,
}

impl<'a> URI<'a> {
    /// Constructs a new URI from a given string. The URI is assumed to be an
    /// absolute, well formed URI.
    pub fn new<T: Into<Cow<'a, str>>>(uri: T) -> URI<'a> {
        let uri = uri.into();
        let qmark = uri.find('?');
        let hmark = uri.find('#');

        let end = uri.len();
        let (path, query, fragment) = match (qmark, hmark) {
            (Some(i), Some(j)) => ((0, i), Some((i+1, j)), Some((j+1, end))),
            (Some(i), None) => ((0, i), Some((i+1, end)), None),
            (None, Some(j)) => ((0, j), None, Some((j+1, end))),
            (None, None) => ((0, end), None, None),
        };

        URI {
            uri: uri,
            path: path,
            query: query,
            fragment: fragment,
            segment_count: AtomicIsize::new(EMPTY),
        }
    }

    /// Returns the number of segments in the URI. Empty segments, which are
    /// invalid according to RFC#3986, are not counted.
    ///
    /// The segment count is cached after the first invocation. As a result,
    /// this function is O(1) after the first invocation, and O(n) before.
    ///
    /// ### Examples
    ///
    /// A valid URI with only non-empty segments:
    ///
    /// ```rust
    /// use rocket::http::uri::URI;
    ///
    /// let uri = URI::new("/a/b/c");
    /// assert_eq!(uri.segment_count(), 3);
    /// ```
    ///
    /// A URI with empty segments:
    ///
    /// ```rust
    /// use rocket::http::uri::URI;
    ///
    /// let uri = URI::new("/a/b//c/d///e");
    /// assert_eq!(uri.segment_count(), 5);
    /// ```
    #[inline(always)]
    pub fn segment_count(&self) -> usize {
        let count = self.segment_count.load(Ordering::Relaxed);
        if count == EMPTY {
            let real_count = self.segments().count();
            if real_count <= isize::max_value() as usize {
                self.segment_count.store(real_count as isize, Ordering::Relaxed);
            }

            real_count
        } else {
            count as usize
        }
    }

    /// Returns an iterator over the segments of the path in this URI. Skips
    /// empty segments.
    ///
    /// ### Examples
    ///
    /// A valid URI with only non-empty segments:
    ///
    /// ```rust
    /// use rocket::http::uri::URI;
    ///
    /// let uri = URI::new("/a/b/c?a=true#done");
    /// for (i, segment) in uri.segments().enumerate() {
    ///     match i {
    ///         0 => assert_eq!(segment, "a"),
    ///         1 => assert_eq!(segment, "b"),
    ///         2 => assert_eq!(segment, "c"),
    ///         _ => panic!("only three segments")
    ///     }
    /// }
    /// ```
    ///
    /// A URI with empty segments:
    ///
    /// ```rust
    /// use rocket::http::uri::URI;
    ///
    /// let uri = URI::new("///a//b///c////d?#");
    /// for (i, segment) in uri.segments().enumerate() {
    ///     match i {
    ///         0 => assert_eq!(segment, "a"),
    ///         1 => assert_eq!(segment, "b"),
    ///         2 => assert_eq!(segment, "c"),
    ///         3 => assert_eq!(segment, "d"),
    ///         _ => panic!("only four segments")
    ///     }
    /// }
    /// ```
    #[inline(always)]
    pub fn segments(&self) -> Segments {
        Segments(self.path())
    }

    /// Returns the path part of this URI.
    ///
    /// ### Examples
    ///
    /// A URI with only a path:
    ///
    /// ```rust
    /// use rocket::http::uri::URI;
    ///
    /// let uri = URI::new("/a/b/c");
    /// assert_eq!(uri.path(), "/a/b/c");
    /// ```
    ///
    /// A URI with other components:
    ///
    /// ```rust
    /// use rocket::http::uri::URI;
    ///
    /// let uri = URI::new("/a/b/c?name=bob#done");
    /// assert_eq!(uri.path(), "/a/b/c");
    /// ```
    #[inline(always)]
    pub fn path(&self) -> &str {
        let (i, j) = self.path;
        &self.uri[i..j]
    }

    /// Returns the query part of this URI without the question mark, if there is
    /// any.
    ///
    /// ### Examples
    ///
    /// A URI with a query part:
    ///
    /// ```rust
    /// use rocket::http::uri::URI;
    ///
    /// let uri = URI::new("/a/b/c?alphabet=true");
    /// assert_eq!(uri.query(), Some("alphabet=true"));
    /// ```
    ///
    /// A URI without the query part:
    ///
    /// ```rust
    /// use rocket::http::uri::URI;
    ///
    /// let uri = URI::new("/a/b/c");
    /// assert_eq!(uri.query(), None);
    /// ```
    #[inline(always)]
    pub fn query(&self) -> Option<&str> {
        self.query.map(|(i, j)| &self.uri[i..j])
    }

    /// Returns the fargment part of this URI without the hash mark, if there is
    /// any.
    ///
    /// ### Examples
    ///
    /// A URI with a fragment part:
    ///
    /// ```rust
    /// use rocket::http::uri::URI;
    ///
    /// let uri = URI::new("/a?alphabet=true#end");
    /// assert_eq!(uri.fragment(), Some("end"));
    /// ```
    ///
    /// A URI without the fragment part:
    ///
    /// ```rust
    /// use rocket::http::uri::URI;
    ///
    /// let uri = URI::new("/a?query=true");
    /// assert_eq!(uri.fragment(), None);
    /// ```
    #[inline(always)]
    pub fn fragment(&self) -> Option<&str> {
        self.fragment.map(|(i, j)| &self.uri[i..j])
    }

    /// Returns a URL-decoded version of the string. If the percent encoded
    /// values are not valid UTF-8, an `Err` is returned.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rocket::http::uri::URI;
    ///
    /// let uri = URI::new("/Hello%2C%20world%21");
    /// let decoded_path = URI::percent_decode(uri.path().as_bytes()).expect("decoded");
    /// assert_eq!(decoded_path, "/Hello, world!");
    /// ```
    pub fn percent_decode(string: &[u8]) -> Result<Cow<str>, Utf8Error> {
        let decoder = url::percent_encoding::percent_decode(string);
        decoder.decode_utf8()
    }

    /// Returns a URL-decoded version of the path. Any invalid UTF-8
    /// percent-encoded byte sequences will be replaced � U+FFFD, the
    /// replacement character.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rocket::http::uri::URI;
    ///
    /// let uri = URI::new("/Hello%2C%20world%21");
    /// let decoded_path = URI::percent_decode_lossy(uri.path().as_bytes());
    /// assert_eq!(decoded_path, "/Hello, world!");
    /// ```
    pub fn percent_decode_lossy(string: &[u8]) -> Cow<str> {
        let decoder = url::percent_encoding::percent_decode(string);
        decoder.decode_utf8_lossy()
    }

    /// Returns a URL-encoded version of the string. Any characters outside of
    /// visible ASCII-range are encoded as well as ' ', '"', '#', '<', '>', '`',
    /// '?', '{', '}', '%', and '/'.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rocket::http::uri::URI;
    ///
    /// let encoded = URI::percent_encode("hello?a=<b>hi</b>");
    /// assert_eq!(encoded, "hello%3Fa=%3Cb%3Ehi%3C%2Fb%3E");
    /// ```
    pub fn percent_encode(string: &str) -> Cow<str> {
        let set = url::percent_encoding::PATH_SEGMENT_ENCODE_SET;
        url::percent_encoding::utf8_percent_encode(string, set).into()
    }

    /// Returns the inner string of this URI.
    ///
    /// The returned string is in raw form. It contains empty segments. If you'd
    /// like a string without empty segments, use `to_string` instead.
    ///
    /// ### Example
    ///
    /// ```rust
    /// use rocket::http::uri::URI;
    ///
    /// let uri = URI::new("/a/b///c/d/e//f?name=Mike#end");
    /// assert_eq!(uri.as_str(), "/a/b///c/d/e//f?name=Mike#end");
    /// ```
    #[inline(always)]
    pub fn as_str(&self) -> &str {
        &self.uri
    }
}

impl<'a> Clone for URI<'a> {
    #[inline(always)]
    fn clone(&self) -> URI<'a> {
        URI {
            uri: self.uri.clone(),
            path: self.path,
            query: self.query,
            fragment: self.fragment,
            segment_count: AtomicIsize::new(EMPTY),
        }
    }
}

impl<'a, 'b> PartialEq<URI<'b>> for URI<'a> {
    #[inline]
    fn eq(&self, other: &URI<'b>) -> bool {
        self.path() == other.path() &&
            self.query() == other.query() &&
            self.fragment() == other.fragment()
    }
}

impl<'a> Eq for URI<'a> {}

impl<'a> From<&'a str> for URI<'a> {
    #[inline(always)]
    fn from(uri: &'a str) -> URI<'a> {
        URI::new(uri)
    }
}

impl From<String> for URI<'static> {
    #[inline(always)]
    fn from(uri: String) -> URI<'static> {
        URI::new(uri)
    }
}

impl<'a> fmt::Display for URI<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // If this is the root path, then there are "zero" segments.
        if self.segment_count() == 0 {
            write!(f, "/")?;
        } else {
            for segment in self.segments() {
                write!(f, "/{}", segment)?;
            }
        }

        if let Some(query_str) = self.query() {
            write!(f, "?{}", query_str)?;
        }

        if let Some(fragment_str) = self.fragment() {
            write!(f, "#{}", fragment_str)?;
        }

        Ok(())
    }
}

/// Trait implemented by types that can be displayed as part of a URI.
///
/// Types implementing this trait can be displayed in a URI-safe manner. Unlike
/// `Display`, the string written by a `UriDisplay` implementation must be
/// URI-safe. In practice, this means that the string must either be
/// percent-encoded or consist only of characters that are alphanumeric, "-",
/// ".", "_", or "~" - the "unreserved" characters.
///
/// # Code Generation
///
/// When the `uri!` macro is used to generate a URI for a route, the types for
/// the route's URI parameters must implement `UriDisplay`. The `UriDisplay`
/// implementation for these types is used when generating the URI.
///
/// To illustrate `UriDisplay`'s role in code generation for `uri!`, consider
/// the following fictional route and struct definition:
///
/// ```rust,ignore
/// struct Value { .. };
///
/// #[get("/item/<id>/<value>")]
/// fn get_item(id: i32, value: Value) -> T { .. }
/// ```
///
/// A URI for this route can be generated as follows:
///
/// ```rust,ignore
/// // With unnamed parameters.
/// uri!(get_item: 100, Value { .. });
///
/// // With named parameters.
/// uri!(get_item: id = 100, value = Value { .. });
/// ```
///
/// After verifying parameters and their types, Rocket will generate code
/// similar to the following:
///
/// ```rust,ignore
/// format!("/item/{id}/{value}",
///     id = &100 as &UriDisplay,
///     value = &Value { .. } as &UriDisplay);
/// ```
///
/// For this expression  to typecheck, both `i32` and `Value` must implement
/// `UriDisplay`. As can be seen, the implementation will be used to display the
/// value in a URI-safe manner.
///
/// [`uri!`]: /rocket_codegen/#procedural-macros
///
/// # Provided Implementations
///
/// Rocket implements `UriDisplay` for several built-in types. Their behavior is
/// documented here.
///
///   * **i8, i16, i32, i64, isize, u8, u16, u32, u64, usize, f32, f64, bool,
///     IpAddr, Ipv4Addr, Ipv6Addr**
///
///     The implementation of `UriDisplay` for these types is identical to the
///     `Display` implementation.
///
///   * **[`&RawStr`](/rocket/http/struct.RawStr.html), String, &str, Cow<str>**
///
///     The string is percent encoded.
///
///   * **&T, &mut T** _where_ **T: UriDisplay**
///
///     Uses the implementation of `UriDisplay` for `T`.
///
/// # Implementing
///
/// Implementing `UriDisplay` is similar to implementing `Display` with the
/// caveat that extra care must be taken to ensure that the written string is
/// URI-safe. As mentioned before, in practice, this means that the string must
/// either be percent-encoded or consist only of characters that are
/// alphanumeric, "-", ".", "_", or "~".
///
/// When manually implementing `UriDisplay` for your types, you should defer to
/// existing implementations of `UriDisplay` as much as possible. In the example
/// below, for instance, `Name`'s implementation defers to `String`'s
/// implementation. To percent-encode a string, use [`URI::percent_encode()`].
///
/// [`URI::percent_encode()`]: https://api.rocket.rs/rocket/http/uri/struct.URI.html#method.percent_encode
///
/// ## Example
///
/// The following snippet consists of a `Name` type that implements both
/// `FromParam` and `UriDisplay`. The `FromParam` implementation allows `Name`
/// to be used as the target type of a dynamic parameter, while the `UriDisplay`
/// implementation allows URIs to be generated for routes with `Name` as a
/// dynamic parameter type.
///
/// ```rust
/// # #![feature(plugin, decl_macro)]
/// # #![plugin(rocket_codegen)]
/// # extern crate rocket;
/// # fn main() {  }
/// use rocket::http::RawStr;
/// use rocket::request::FromParam;
///
/// struct Name(String);
///
/// impl<'r> FromParam<'r> for Name {
///     type Error = &'r RawStr;
///
///     /// Validates parameters that contain no spaces.
///     fn from_param(param: &'r RawStr) -> Result<Self, Self::Error> {
///         let decoded = param.percent_decode().map_err(|_| param)?;
///         match decoded.contains(' ') {
///             false => Ok(Name(decoded.into_owned())),
///             true => Err(param),
///         }
///     }
/// }
///
/// use std::fmt;
/// use rocket::http::uri::UriDisplay;
/// use rocket::response::Redirect;
///
/// impl UriDisplay for Name {
///     /// Delegates to the `UriDisplay` implementation for `String` to ensure
///     /// that the written string is URI-safe. In this case, the string will
///     /// be percent encoded.
///     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
///         UriDisplay::fmt(&self.0, f)
///     }
/// }
///
/// #[get("/name/<name>")]
/// fn redirector(name: Name) -> Redirect {
///     Redirect::to(uri!(real: name).as_str())
/// }
///
/// #[get("/<name>")]
/// fn real(name: Name) -> String {
///     format!("Hello, {}!", name.0)
/// }
/// ```

// FIXME: Put this more narrative-like text in the guide. Fix it up beforehand.
//
// Now we have the following routes. The first route accepts a URI parameter of
// type `Name` and redirects to the second route:
//
// ```rust
// # #![feature(plugin, decl_macro)]
// # #![plugin(rocket_codegen)]
// # extern crate rocket;
// # use rocket::request::FromParam;
// # use rocket::http::RawStr;
// # struct Name(String);
// # impl Name {
// #     fn new(name: String) -> Option<Name> {
// #       if !name.contains(' ') { Some(name) } else { None }
// #     }
// # }
// # impl<'r> FromParam<'r> for Name {
// #     type Error = &'r RawStr;
// #     fn from_param(param: &'r RawStr) -> Result<Self, Self::Error> {
// #         Name::new(param.percent_decode().into_owned()).ok_or(param)
// #     }
// # }
// use rocket::response::Redirect;
//
// #[get("/name/<name>")]
// fn redirector(name: Name) -> Redirect {
//     Redirect::to(&format!("/{}", name.0))
// }
//
// #[get("/<name>")]
// fn real(name: Name) -> String {
//     format!("Hello, {}!", name.0)
// }
// ```
//
// The redirection in the `redirector` route creates a URI that should lead to
// the `real` route. But it does this in an ad-hoc manner. What happens if the
// `real` route changes? At best, the redirection will fail and the user will
// receive a 404.
//
// To prevent this kind of issue the `uri!` macro can be used, passing in the
// `name` received from the route. When the `Name` type is used along with the
// `uri!` macro, the `UriDisplay` trait must be implemented. Both of these
// steps are done in the example below:
//
// ```rust
// # #![feature(plugin, decl_macro)]
// # #![plugin(rocket_codegen)]
// # extern crate rocket;
// # use rocket::request::FromParam;
// # use rocket::http::RawStr;
// # struct Name(String);
// # impl Name {
// #     fn new(name: String) -> Option<Name> {
// #       if !name.contains(' ') { Some(name) } else { None }
// #     }
// # }
// # impl<'r> FromParam<'r> for Name {
// #     type Error = &'r RawStr;
// #     fn from_param(param: &'r RawStr) -> Result<Self, Self::Error> {
// #         Name::new(param.percent_decode().into_owned()).ok_or(param)
// #     }
// # }
// use std::fmt;
// use rocket::http::uri::UriDisplay;
// use rocket::response::Redirect;
//
// impl UriDisplay for Name {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         UriDisplay::fmt(&self.0, f)
//     }
// }
//
// #[get("/name/<name>")]
// fn redirector(name: Name) -> Redirect {
//     Redirect::to(uri!(real: name).as_str())
// }
//
// #[get("/<name>")]
// fn real(name: Name) -> String {
//     format!("Hello, {}!", name.0)
// }
// ```
pub trait UriDisplay {
    /// Formats `self` in a URI-safe manner using the given formatter.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result;
}

impl<'a> fmt::Display for &'a UriDisplay {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        UriDisplay::fmt(*self, f)
    }
}

/// Percent-encodes the raw string.
impl<'a> UriDisplay for &'a RawStr {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", URI::percent_encode((*self).as_str()))
    }
}

/// Percent-encodes the raw string.
impl<'a> UriDisplay for &'a str {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", URI::percent_encode(self))
    }
}

/// Percent-encodes the raw string.
impl<'a> UriDisplay for Cow<'a, str> {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", URI::percent_encode(self))
    }
}

/// Percent-encodes the raw string.
impl UriDisplay for String {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", URI::percent_encode(self.as_str()))
    }
}

macro_rules! impl_with_display {
    ($($T:ty),+) => {$(
        /// This implementation is identical to the `Display` implementation.
        impl UriDisplay for $T  {
            #[inline(always)]
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                fmt::Display::fmt(self, f)
            }
        }
    )+}
}

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

impl_with_display! {
    i8, i16, i32, i64, isize, u8, u16, u32, u64, usize, f32, f64, bool,
    IpAddr, Ipv4Addr, Ipv6Addr
}

macro_rules! impl_for_ref {
    ($($T:ty),+) => {$(
        /// Uses the implementation of `UriDisplay` for `T`.
        impl<'a, T: UriDisplay + ?Sized> UriDisplay for $T {
            #[inline(always)]
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                UriDisplay::fmt(*self, f)
            }
        }
    )+}
}

impl_for_ref!(&'a mut T, &'a T);

/// Iterator over the segments of an absolute URI path. Skips empty segments.
///
/// ### Examples
///
/// ```rust
/// use rocket::http::uri::URI;
///
/// let uri = URI::new("/a/////b/c////////d");
/// let segments = uri.segments();
/// for (i, segment) in segments.enumerate() {
///     match i {
///         0 => assert_eq!(segment, "a"),
///         1 => assert_eq!(segment, "b"),
///         2 => assert_eq!(segment, "c"),
///         3 => assert_eq!(segment, "d"),
///         _ => panic!("only four segments")
///     }
/// }
/// ```
#[derive(Clone, Debug)]
pub struct Segments<'a>(pub &'a str);

impl<'a> Iterator for Segments<'a> {
    type Item = &'a str;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // Find the start of the next segment (first that's not '/').
        let i = match self.0.find(|c| c != '/') {
            Some(index) => index,
            None => return None,
        };

        // Get the index of the first character that _is_ a '/' after start.
        // j = index of first character after i (hence the i +) that's not a '/'
        let j = self.0[i..].find('/').map_or(self.0.len(), |j| i + j);

        // Save the result, update the iterator, and return!
        let result = Some(&self.0[i..j]);
        self.0 = &self.0[j..];
        result
    }

    // TODO: Potentially take a second parameter with Option<cached count> and
    // return it here if it's Some. The downside is that a decision has to be
    // made about -when- to compute and cache that count. A place to do it is in
    // the segments() method. But this means that the count will always be
    // computed regardless of whether it's needed. Maybe this is ok. We'll see.
    // fn count(self) -> usize where Self: Sized {
    //     self.1.unwrap_or_else(self.fold(0, |cnt, _| cnt + 1))
    // }
}

/// Errors which can occur when attempting to interpret a segment string as a
/// valid path segment.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SegmentError {
    /// The segment contained invalid UTF8 characters when percent decoded.
    Utf8(Utf8Error),
    /// The segment started with the wrapped invalid character.
    BadStart(char),
    /// The segment contained the wrapped invalid character.
    BadChar(char),
    /// The segment ended with the wrapped invalid character.
    BadEnd(char),
}

#[cfg(test)]
mod tests {
    use super::URI;

    fn seg_count(path: &str, expected: usize) -> bool {
        let actual = URI::new(path).segment_count();
        if actual != expected {
            trace_!("Count mismatch: expected {}, got {}.", expected, actual);
            trace_!("{}", if actual != expected { "lifetime" } else { "buf" });
            trace_!("Segments (for {}):", path);
            for (i, segment) in URI::new(path).segments().enumerate() {
                trace_!("{}: {}", i, segment);
            }
        }

        actual == expected
    }

    fn eq_segments(path: &str, expected: &[&str]) -> bool {
        let uri = URI::new(path);
        let actual: Vec<&str> = uri.segments().collect();
        actual == expected
    }

    #[test]
    fn send_and_sync() {
        fn assert<T: Send + Sync>() {};
        assert::<URI>();
    }

    #[test]
    fn simple_segment_count() {
        assert!(seg_count("", 0));
        assert!(seg_count("/", 0));
        assert!(seg_count("a", 1));
        assert!(seg_count("/a", 1));
        assert!(seg_count("a/", 1));
        assert!(seg_count("/a/", 1));
        assert!(seg_count("/a/b", 2));
        assert!(seg_count("/a/b/", 2));
        assert!(seg_count("a/b/", 2));
        assert!(seg_count("ab/", 1));
    }

    #[test]
    fn segment_count() {
        assert!(seg_count("////", 0));
        assert!(seg_count("//a//", 1));
        assert!(seg_count("//abc//", 1));
        assert!(seg_count("//abc/def/", 2));
        assert!(seg_count("//////abc///def//////////", 2));
        assert!(seg_count("a/b/c/d/e/f/g", 7));
        assert!(seg_count("/a/b/c/d/e/f/g", 7));
        assert!(seg_count("/a/b/c/d/e/f/g/", 7));
        assert!(seg_count("/a/b/cdjflk/d/e/f/g", 7));
        assert!(seg_count("//aaflja/b/cdjflk/d/e/f/g", 7));
        assert!(seg_count("/a   /b", 2));
    }

    #[test]
    fn single_segments_match() {
        assert!(eq_segments("", &[]));
        assert!(eq_segments("a", &["a"]));
        assert!(eq_segments("/a", &["a"]));
        assert!(eq_segments("/a/", &["a"]));
        assert!(eq_segments("a/", &["a"]));
        assert!(eq_segments("///a/", &["a"]));
        assert!(eq_segments("///a///////", &["a"]));
        assert!(eq_segments("a///////", &["a"]));
        assert!(eq_segments("//a", &["a"]));
        assert!(eq_segments("", &[]));
        assert!(eq_segments("abc", &["abc"]));
        assert!(eq_segments("/a", &["a"]));
        assert!(eq_segments("/abc/", &["abc"]));
        assert!(eq_segments("abc/", &["abc"]));
        assert!(eq_segments("///abc/", &["abc"]));
        assert!(eq_segments("///abc///////", &["abc"]));
        assert!(eq_segments("abc///////", &["abc"]));
        assert!(eq_segments("//abc", &["abc"]));
    }

    #[test]
    fn multi_segments_match() {
        assert!(eq_segments("a/b/c", &["a", "b", "c"]));
        assert!(eq_segments("/a/b", &["a", "b"]));
        assert!(eq_segments("/a///b", &["a", "b"]));
        assert!(eq_segments("a/b/c/d", &["a", "b", "c", "d"]));
        assert!(eq_segments("///a///////d////c", &["a", "d", "c"]));
        assert!(eq_segments("abc/abc", &["abc", "abc"]));
        assert!(eq_segments("abc/abc/", &["abc", "abc"]));
        assert!(eq_segments("///abc///////a", &["abc", "a"]));
        assert!(eq_segments("/////abc/b", &["abc", "b"]));
        assert!(eq_segments("//abc//c////////d", &["abc", "c", "d"]));
    }

    #[test]
    fn multi_segments_match_funky_chars() {
        assert!(eq_segments("a/b/c!!!", &["a", "b", "c!!!"]));
        assert!(eq_segments("a  /b", &["a  ", "b"]));
        assert!(eq_segments("  a/b", &["  a", "b"]));
        assert!(eq_segments("  a/b  ", &["  a", "b  "]));
        assert!(eq_segments("  a///b  ", &["  a", "b  "]));
        assert!(eq_segments("  ab  ", &["  ab  "]));
    }

    #[test]
    fn segment_mismatch() {
        assert!(!eq_segments("", &["a"]));
        assert!(!eq_segments("a", &[]));
        assert!(!eq_segments("/a/a", &["a"]));
        assert!(!eq_segments("/a/b", &["b", "a"]));
        assert!(!eq_segments("/a/a/b", &["a", "b"]));
        assert!(!eq_segments("///a/", &[]));
    }

    fn test_query(uri: &str, query: Option<&str>) {
        let uri = URI::new(uri);
        assert_eq!(uri.query(), query);
    }

    fn test_fragment(uri: &str, fragment: Option<&str>) {
        let uri = URI::new(uri);
        assert_eq!(uri.fragment(), fragment);
    }

    #[test]
    fn query_does_not_exist() {
        test_query("/test", None);
        test_query("/a/b/c/d/e", None);
        test_query("/////", None);
        test_query("//a///", None);
    }

    #[test]
    fn query_exists() {
        test_query("/test?abc", Some("abc"));
        test_query("/a/b/c?abc", Some("abc"));
        test_query("/a/b/c/d/e/f/g/?abc#hijklmnop", Some("abc"));
        test_query("?123", Some("123"));
        test_query("?", Some(""));
        test_query("/?", Some(""));
        test_query("?#", Some(""));
        test_query("/?hi", Some("hi"));
    }

    #[test]
    fn fragment_exists() {
        test_fragment("/test#abc", Some("abc"));
        test_fragment("/#abc", Some("abc"));
        test_fragment("/a/b/c?123#a", Some("a"));
        test_fragment("/#a", Some("a"));
    }

    #[test]
    fn fragment_does_not_exist() {
        test_fragment("/testabc", None);
        test_fragment("/abc", None);
        test_fragment("/a/b/c?123", None);
        test_fragment("/a", None);
    }

    #[test]
    fn to_string() {
        let uri_to_string = |string| URI::new(string).to_string();

        assert_eq!(uri_to_string("/"), "/".to_string());
        assert_eq!(uri_to_string("//"), "/".to_string());
        assert_eq!(uri_to_string("//////a/"), "/a".to_string());
        assert_eq!(uri_to_string("//ab"), "/ab".to_string());
        assert_eq!(uri_to_string("//a"), "/a".to_string());
        assert_eq!(uri_to_string("/a/b///c"), "/a/b/c".to_string());
        assert_eq!(uri_to_string("/a///b/c/d///"), "/a/b/c/d".to_string());
    }
}
