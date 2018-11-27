use std::{fmt, path};
use std::borrow::Cow;

use percent_encoding::utf8_percent_encode;

use uri::{Uri, UriPart, Path, Query, Formatter, UNSAFE_PATH_ENCODE_SET};
use {RawStr, ext::Normalize};

/// Trait implemented by types that can be displayed as part of a URI in `uri!`.
///
/// Types implementing this trait can be displayed in a URI-safe manner. Unlike
/// `Display`, the string written by a `UriDisplay` implementation must be
/// URI-safe. In practice, this means that the string must either be
/// percent-encoded or consist only of characters that are alphanumeric, "-",
/// ".", "_", or "~" - the "unreserved" characters.
///
/// # Marker Generic: `UriDisplay<Path>` vs. `UriDisplay<Query>`
///
/// The [`UriPart`] parameter `P` in `UriDisplay<P>` must be either [`Path`] or
/// [`Query`] (see the [`UriPart`] documentation for how this is enforced),
/// resulting in either `UriDisplay<Path>` or `UriDisplay<Query>`.
///
/// As the names might imply, the `Path` version of the trait is used when
/// displaying parameters in the path part of the URI while the `Query` version
/// is used when display parameters in the query part of the URI. These distinct
/// versions of the trait exist exactly to differentiate, at the type-level,
/// where in the URI a value is to be written to, allowing for type safety in
/// the face of differences between the two locations. For example, while it is
/// valid to use a value of `None` in the query part, omitting the parameter
/// entirely, doing so is _not_ valid in the path part. By differentiating in
/// the type system, both of these conditions can be enforced appropriately
/// through distinct implementations of `UriDisplay<Path>` and
/// `UriDisplay<Query>`.
///
/// Occasionally, the implementation of `UriDisplay` is independent of where the
/// parameter is to be displayed. When this is the case, the parameter may be
/// kept generic. That is, implementations can take the form:
///
/// ```rust
/// # extern crate rocket;
/// # use std::fmt;
/// # use rocket::http::uri::{UriPart, UriDisplay, Formatter};
/// # struct SomeType;
/// impl<P: UriPart> UriDisplay<P> for SomeType
/// # { fn fmt(&self, f: &mut Formatter<P>) -> fmt::Result { Ok(()) } }
/// ```
///
/// [`UriPart`]: uri::UriPart
/// [`Path`]: uri::Path
/// [`Query`]: uri::Query
///
/// # Code Generation
///
/// When the `uri!` macro is used to generate a URI for a route, the types for
/// the route's _path_ URI parameters must implement `UriDisplay<Path>`, while
/// types in the route's query parameters must implement `UriDisplay<Query>`.
/// The `UriDisplay` implementation for these types is used when generating the
/// URI.
///
/// To illustrate `UriDisplay`'s role in code generation for `uri!`, consider
/// the following route:
///
/// ```rust
/// # #![feature(proc_macro_hygiene, decl_macro)]
/// # #[macro_use] extern crate rocket;
/// #[get("/item/<id>?<track>")]
/// fn get_item(id: i32, track: String) { /* .. */ }
/// ```
///
/// A URI for this route can be generated as follows:
///
/// ```rust
/// # #![feature(proc_macro_hygiene, decl_macro)]
/// # #[macro_use] extern crate rocket;
/// # type T = ();
/// # #[get("/item/<id>?<track>")]
/// # fn get_item(id: i32, track: String) { /* .. */ }
/// #
/// // With unnamed parameters.
/// uri!(get_item: 100, "inbound");
///
/// // With named parameters.
/// uri!(get_item: id = 100, track = "inbound");
/// uri!(get_item: track = "inbound", id = 100);
/// ```
///
/// After verifying parameters and their types, Rocket will generate code
/// similar (in spirit) to the following:
///
/// ```rust
/// # extern crate rocket;
/// # use rocket::http::uri::{UriDisplay, Path, Query, Origin};
/// #
/// Origin::parse(&format!("/item/{}?track={}",
///     &100 as &UriDisplay<Path>, &"inbound" as &UriDisplay<Query>));
/// ```
///
/// For this expression to typecheck, `i32` must implement `UriDisplay<Path>`
/// and `Value` must implement `UriDisplay<Query>`. As can be seen, the
/// implementations will be used to display the value in a URI-safe manner.
///
/// [`uri!`]: /rocket_codegen/#typed-uris-uri
///
/// # Provided Implementations
///
/// Rocket implements `UriDisplay<P>` for all `P: UriPart` for several built-in
/// types.
///
///   * **i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, f32,
///     f64, bool, IpAddr, Ipv4Addr, Ipv6Addr**
///
///     The implementation of `UriDisplay` for these types is identical to the
///     `Display` implementation.
///
///   * **[`&RawStr`](RawStr), `String`, `&str`, `Cow<str>`**
///
///     The string is percent encoded.
///
///   * **`&T`, `&mut T`** _where_ **`T: UriDisplay`**
///
///     Uses the implementation of `UriDisplay` for `T`.
///
/// Rocket implements `UriDisplay<Path>` (but not `UriDisplay<Query>`) for
/// several built-in types.
///
///   * `T` for **`Option<T>`** _where_ **`T: UriDisplay<Path>`**
///
///     Uses the implementation of `UriDisplay` for `T::Target`.
///
///     When a type of `Option<T>` appears in a route path, use a type of `T` as
///     the parameter in `uri!`. Note that `Option<T>` itself _does not_
///     implement `UriDisplay<Path>`.
///
///   * `T` for **`Result<T, E>`** _where_ **`T: UriDisplay<Path>`**
///
///     Uses the implementation of `UriDisplay` for `T::Target`.
///
///     When a type of `Result<T, E>` appears in a route path, use a type of `T`
///     as the parameter in `uri!`. Note that `Result<T, E>` itself _does not_
///     implement `UriDisplay<Path>`.
///
/// Rocket implements `UriDisplay<Query>` (but not `UriDisplay<Path>`) for
/// several built-in types.
///
///   * **`Form<T>`, `LenientForm<T>`** _where_ **`T: FromUriParam + FromForm`**
///
///     Uses the implementation of `UriDisplay` for `T::Target`.
///
///     In general, when a type of `Form<T>` is to be displayed as part of a
///     URI's query, it suffices to derive `UriDisplay` for `T`. Note that any
///     type that can be converted into a `T` using [`FromUriParam`] can be used
///     in place of a `Form<T>` in a `uri!` invocation.
///
///   * **`Option<T>`** _where_ **`T: UriDisplay<Query>`**
///
///     If the `Option` is `Some`, uses the implementation of `UriDisplay` for
///     `T`. Otherwise, nothing is rendered.
///
///   * **`Result<T, E>`** _where_ **`T: UriDisplay<Query>`**
///
///     If the `Result` is `Ok`, uses the implementation of `UriDisplay` for
///     `T`. Otherwise, nothing is rendered.
///
/// # Deriving
///
/// Manually implementing `UriDisplay` should be done with care. For most use
/// cases, deriving `UriDisplay` will suffice:
///
/// ```rust
/// # #[macro_use] extern crate rocket;
/// # use rocket::http::uri::{UriDisplay, Query, Path};
/// // Derives `UriDisplay<Query>`
/// #[derive(UriDisplayQuery)]
/// struct User {
///     name: String,
///     age: usize,
/// }
///
/// let user = User { name: "Michael Smith".into(), age: 31 };
/// let uri_string = format!("{}", &user as &UriDisplay<Query>);
/// assert_eq!(uri_string, "name=Michael%20Smith&age=31");
///
/// // Derives `UriDisplay<Path>`
/// #[derive(UriDisplayPath)]
/// struct Name(String);
///
/// let name = Name("Bob Smith".into());
/// let uri_string = format!("{}", &name as &UriDisplay<Path>);
/// assert_eq!(uri_string, "Bob%20Smith");
/// ```
///
/// As long as every field in the structure (or enum) implements `UriDisplay`,
/// the trait can be derived. The implementation calls
/// [`Formatter::write_named_value()`] for every named field and
/// [`Formatter::write_value()`] for every unnamed field. See the [`UriDisplay`
/// derive] documentation for full details.
///
/// [`UriDisplay` derive]: ../../../rocket_codegen/derive.UriDisplay.html
/// [`Formatter::write_named_value()`]: uri::Formatter::write_named_value()
/// [`Formatter::write_value()`]: uri::Formatter::write_value()
///
/// # Implementing
///
/// Implementing `UriDisplay` is similar to implementing
/// [`Display`](::std::fmt::Display) with the caveat that extra care must be
/// taken to ensure that the written string is URI-safe. As mentioned before, in
/// practice, this means that the string must either be percent-encoded or
/// consist only of characters that are alphanumeric, "-", ".", "_", or "~".
///
/// When manually implementing `UriDisplay` for your types, you should defer to
/// existing implementations of `UriDisplay` as much as possible. In the example
/// below, for instance, `Name`'s implementation defers to `String`'s
/// implementation. To percent-encode a string, use
/// [`Uri::percent_encode()`](uri::Uri::percent_encode()).
///
/// ## Example
///
/// The following snippet consists of a `Name` type that implements both
/// `FromParam` and `UriDisplay<Path>`. The `FromParam` implementation allows
/// `Name` to be used as the target type of a dynamic parameter, while the
/// `UriDisplay` implementation allows URIs to be generated for routes with
/// `Name` as a dynamic path parameter type. Note the custom parsing in the
/// `FromParam` implementation; as a result of this, a custom (reflexive)
/// `UriDisplay` implementation is required.
///
/// ```rust
/// # #![feature(proc_macro_hygiene, decl_macro)]
/// # #[macro_use] extern crate rocket;
/// use rocket::http::RawStr;
/// use rocket::request::FromParam;
///
/// struct Name(String);
///
/// const PREFIX: &str = "name:";
///
/// impl<'r> FromParam<'r> for Name {
///     type Error = &'r RawStr;
///
///     /// Validates parameters that start with 'name:', extracting the text
///     /// after 'name:' as long as there is at least one character.
///     fn from_param(param: &'r RawStr) -> Result<Self, Self::Error> {
///         let decoded = param.percent_decode().map_err(|_| param)?;
///         if !decoded.starts_with(PREFIX) || decoded.len() < (PREFIX.len() + 1) {
///             return Err(param);
///         }
///
///         let real_name = decoded[PREFIX.len()..].to_string();
///         Ok(Name(real_name))
///     }
/// }
///
/// use std::fmt;
/// use rocket::http::uri::{Formatter, UriDisplay, Path};
/// use rocket::response::Redirect;
///
/// impl UriDisplay<Path> for Name {
///     // Delegates to the `UriDisplay` implementation for `String` via the
///     // call to `write_value` to ensure /// that the written string is
///     // URI-safe. In this case, the string will /// be percent encoded.
///     // Prefixes the inner name with `name:`.
///     fn fmt(&self, f: &mut Formatter<Path>) -> fmt::Result {
///         f.write_value(&format!("name:{}", self.0))
///     }
/// }
///
/// #[get("/name/<name>")]
/// fn redirector(name: Name) -> Redirect {
///     Redirect::to(uri!(real: name))
/// }
///
/// #[get("/<name>")]
/// fn real(name: Name) -> String {
///     format!("Hello, {}!", name.0)
/// }
///
/// let uri = uri!(real: Name("Mike Smith".into()));
/// assert_eq!(uri.path(), "/name:Mike%20Smith");
/// ```
pub trait UriDisplay<P: UriPart> {
    /// Formats `self` in a URI-safe manner using the given formatter.
    fn fmt(&self, f: &mut Formatter<P>) -> fmt::Result;
}

impl<'a> fmt::Display for &'a UriDisplay<Path> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        UriDisplay::fmt(*self, &mut <Formatter<Path>>::new(f))
    }
}

impl<'a> fmt::Display for &'a UriDisplay<Query> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        UriDisplay::fmt(*self, &mut <Formatter<Query>>::new(f))
    }
}

/// Percent-encodes the raw string.
impl<P: UriPart> UriDisplay<P> for RawStr {
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter<P>) -> fmt::Result {
        f.write_raw(&Uri::percent_encode(self.as_str()))
    }
}

/// Percent-encodes the raw string.
impl<P: UriPart> UriDisplay<P> for str {
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter<P>) -> fmt::Result {
        f.write_raw(&Uri::percent_encode(self))
    }
}

/// Percent-encodes the raw string.
impl<'a, P: UriPart> UriDisplay<P> for Cow<'a, str> {
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter<P>) -> fmt::Result {
        f.write_raw(&Uri::percent_encode(self))
    }
}

/// Percent-encodes the raw string.
impl<P: UriPart> UriDisplay<P> for String {
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter<P>) -> fmt::Result {
        f.write_raw(&Uri::percent_encode(self.as_str()))
    }
}

/// Percent-encodes each segment in the path and normalizes separators.
impl UriDisplay<Path> for path::PathBuf {
    #[inline]
    fn fmt(&self, f: &mut Formatter<Path>) -> fmt::Result {
        self.as_path().fmt(f)
    }
}

/// Percent-encodes each segment in the path and normalizes separators.
impl UriDisplay<Path> for path::Path {
    #[inline]
    fn fmt(&self, f: &mut Formatter<Path>) -> fmt::Result {
        let string = self.normalized_str();
        let enc: Cow<str> = utf8_percent_encode(&string, UNSAFE_PATH_ENCODE_SET).into();
        f.write_raw(&enc)
    }
}

macro_rules! impl_with_display {
    ($($T:ty),+) => {$(
        /// This implementation is identical to the `Display` implementation.
        impl<P: UriPart> UriDisplay<P> for $T  {
            #[inline(always)]
            fn fmt(&self, f: &mut Formatter<P>) -> fmt::Result {
                use std::fmt::Write;
                write!(f, "{}", self)
            }
        }
    )+}
}

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

impl_with_display! {
    i8, i16, i32, i64, i128, isize,
    u8, u16, u32, u64, u128, usize,
    f32, f64, bool,
    IpAddr, Ipv4Addr, Ipv6Addr
}

macro_rules! impl_for_ref {
    ($($T:ty),+) => {$(
        /// Uses the implementation of `UriDisplay` for `T`.
        impl<'a, P: UriPart, T: UriDisplay<P> + ?Sized> UriDisplay<P> for $T {
            #[inline(always)]
            fn fmt(&self, f: &mut Formatter<P>) -> fmt::Result {
                UriDisplay::fmt(*self, f)
            }
        }
    )+}
}

impl_for_ref!(&'a mut T, &'a T);

impl<T: UriDisplay<Query>> UriDisplay<Query> for Option<T> {
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter<Query>) -> fmt::Result {
        if let Some(v) = self {
            f.write_value(&v)?;
        }

        Ok(())
    }
}

impl<E, T: UriDisplay<Query>> UriDisplay<Query> for Result<T, E> {
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter<Query>) -> fmt::Result {
        if let Ok(v) = self {
            f.write_value(&v)?;
        }

        Ok(())
    }
}
