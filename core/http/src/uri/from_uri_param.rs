use std::path::{Path, PathBuf};

use RawStr;
use uri::{self, UriPart, UriDisplay};

/// Conversion trait for parameters used in [`uri!`] invocations.
///
/// Rocket provides a blanket implementation for all types that implement
/// [`UriDisplay`]. As such, this trait typically does not need to be implemented.
/// Instead, implement [`UriDisplay`].
///
/// # Overview
///
/// This trait is invoked once per expression passed into a [`uri!`] invocation.
/// In particular, for a route URI parameter of type `T` and a user-supplied
/// expression of type `S`, `<T as FromUriParam<S>>::from_uri_param` is
/// invoked. The returned value is used in place of the user's value and
/// rendered using its [`UriDisplay`] implementation.
///
/// This trait allows types that differ from the route URI parameter's types to
/// be used in their place at no cost. For instance, the following
/// implementation, provided by Rocket, allows an `&str` to be used in a `uri!`
/// invocation for route URI parameters declared as `String`:
///
/// ```rust
/// # extern crate rocket;
/// # use rocket::http::uri::{FromUriParam, UriPart};
/// # struct S;
/// # type String = S;
/// impl<'a, P: UriPart> FromUriParam<P, &'a str> for String {
///     type Target = &'a str;
/// #   fn from_uri_param(s: &'a str) -> Self::Target { "hi" }
/// }
/// ```
///
/// Because the [`FromUriParam::Target`] type is the same as the input type, the
/// conversion is a no-op and free of cost, allowing an `&str` to be used in
/// place of a `String` without penalty. A similar no-op conversion exists for
/// [`&RawStr`](RawStr):
///
/// ```rust
/// # extern crate rocket;
/// # use rocket::http::uri::{FromUriParam, UriPart};
/// # struct S;
/// # type RawStr = S;
/// impl<'a, 'b, P: UriPart> FromUriParam<P, &'a str> for &'b RawStr {
///     type Target = &'a str;
/// #   fn from_uri_param(s: &'a str) -> Self::Target { "hi" }
/// }
/// ```
///
/// # Provided Implementations
///
/// See [Foreign Impls](#foreign-impls) for implementations provided by Rocket.
///
/// # Implementing
///
/// This trait should only be implemented when you'd like to allow a type
/// different from the route's declared type to be used in its place in a `uri!`
/// invocation. For instance, if the route has a type of `T` and you'd like to
/// use a type of `S` in a `uri!` invocation, you'd implement `FromUriParam<P,
/// T> for S` where `P` is `Path` for conversions valid in the path part of a
/// URI, `Uri` for conversions valid in the query part of a URI, or `P: UriPart`
/// when a conversion is valid in either case.
///
/// This is typically only warranted for owned-value types with corresponding
/// reference types: `String` and `&str`, for instance. In this case, it's
/// desirable to allow an `&str` to be used in place of a `String`.
///
/// When implementing `FromUriParam`, be aware that Rocket will use the
/// [`UriDisplay`] implementation of [`FromUriParam::Target`], _not_ of the
/// source type. Incorrect implementations can result in creating unsafe URIs.
///
/// # Example
///
/// The following example implements `FromUriParam<Query, (&str, &str)>` for a
/// `User` type. The implementation allows an `(&str, &str)` type to be used in
/// a `uri!` invocation where a `User` type is expected in the query part of the
/// URI.
///
/// ```rust
/// # #[macro_use] extern crate rocket;
/// use std::fmt;
///
/// use rocket::http::RawStr;
/// use rocket::http::uri::{Formatter, UriDisplay, FromUriParam, Query};
///
/// #[derive(FromForm)]
/// struct User<'a> {
///     name: &'a RawStr,
///     nickname: String,
/// }
///
/// impl<'a> UriDisplay<Query> for User<'a> {
///     fn fmt(&self, f: &mut Formatter<Query>) -> fmt::Result {
///         f.write_named_value("name", &self.name)?;
///         f.write_named_value("nickname", &self.nickname)
///     }
/// }
///
/// impl<'a, 'b> FromUriParam<Query, (&'a str, &'b str)> for User<'a> {
///     type Target = User<'a>;
///
///     fn from_uri_param((name, nickname): (&'a str, &'b str)) -> User<'a> {
///         User { name: name.into(), nickname: nickname.to_string() }
///     }
/// }
/// ```
///
/// With these implementations, the following typechecks:
///
/// ```rust
/// # #![feature(proc_macro_hygiene, decl_macro)]
/// # #[macro_use] extern crate rocket;
/// # use std::fmt;
/// use rocket::http::RawStr;
/// use rocket::request::Form;
/// # use rocket::http::uri::{Formatter, UriDisplay, FromUriParam, Query};
/// #
/// # #[derive(FromForm)]
/// # struct User<'a> { name: &'a RawStr, nickname: String, }
/// #
/// # impl<'a> UriDisplay<Query> for User<'a> {
/// #     fn fmt(&self, f: &mut Formatter<Query>) -> fmt::Result {
/// #         f.write_named_value("name", &self.name)?;
/// #         f.write_named_value("nickname", &self.nickname)
/// #     }
/// # }
/// #
/// # impl<'a, 'b> FromUriParam<Query, (&'a str, &'b str)> for User<'a> {
/// #     type Target = User<'a>;
/// #     fn from_uri_param((name, nickname): (&'a str, &'b str)) -> User<'a> {
/// #         User { name: name.into(), nickname: nickname.to_string() }
/// #     }
/// # }
///
/// #[post("/<name>?<user..>")]
/// fn some_route(name: &RawStr, user: Form<User>)  { /* .. */ }
///
/// let uri = uri!(some_route: name = "hey", user = ("Robert Mike", "Bob"));
/// assert_eq!(uri.path(), "/hey");
/// assert_eq!(uri.query(), Some("name=Robert%20Mike&nickname=Bob"));
/// ```
///
/// [`uri!`]: ::rocket_codegen::uri
/// [`UriDisplay`]: uri::UriDisplay
/// [`FromUriParam::Target`]: uri::FromUriParam::Target
pub trait FromUriParam<P: UriPart, T> {
    /// The resulting type of this conversion.
    type Target: UriDisplay<P>;

    /// Converts a value of type `T` into a value of type `Self::Target`. The
    /// resulting value of type `Self::Target` will be rendered into a URI using
    /// its [`UriDisplay`](uri::UriDisplay) implementation.
    fn from_uri_param(param: T) -> Self::Target;
}

impl<P: UriPart, T: UriDisplay<P>> FromUriParam<P, T> for T {
    type Target = T;
    #[inline(always)]
    fn from_uri_param(param: T) -> T { param }
}

impl<'a, P: UriPart, T: UriDisplay<P>> FromUriParam<P, &'a T> for T {
    type Target = &'a T;
    #[inline(always)]
    fn from_uri_param(param: &'a T) -> &'a T { param }
}

impl<'a, P: UriPart, T: UriDisplay<P>> FromUriParam<P, &'a mut T> for T {
    type Target = &'a mut T;
    #[inline(always)]
    fn from_uri_param(param: &'a mut T) -> &'a mut T { param }
}

/// A no cost conversion allowing an `&str` to be used in place of a `String`.
impl<'a, P: UriPart> FromUriParam<P, &'a str> for String {
    type Target = &'a str;
    #[inline(always)]
    fn from_uri_param(param: &'a str) -> &'a str { param }
}

/// A no cost conversion allowing an `&str` to be used in place of an `&RawStr`.
impl<'a, 'b, P: UriPart> FromUriParam<P, &'a str> for &'b RawStr {
    type Target = &'a str;
    #[inline(always)]
    fn from_uri_param(param: &'a str) -> &'a str { param }
}

/// A no cost conversion allowing a `String` to be used in place of an `&RawStr`.
impl<'a, P: UriPart> FromUriParam<P, String> for &'a RawStr {
    type Target = String;
    #[inline(always)]
    fn from_uri_param(param: String) -> String { param }
}

/// A no cost conversion allowing a `String` to be used in place of an `&str`.
impl<'a, P: UriPart> FromUriParam<P, String> for &'a str {
    type Target = String;
    #[inline(always)]
    fn from_uri_param(param: String) -> String { param }
}

/// A no cost conversion allowing an `&Path` to be used in place of a `PathBuf`.
impl<'a> FromUriParam<uri::Path, &'a Path> for PathBuf {
    type Target = &'a Path;
    #[inline(always)]
    fn from_uri_param(param: &'a Path) -> &'a Path { param }
}

/// A no cost conversion allowing an `&str` to be used in place of a `PathBuf`.
impl<'a> FromUriParam<uri::Path, &'a str> for PathBuf {
    type Target = &'a Path;

    #[inline(always)]
    fn from_uri_param(param: &'a str) -> &'a Path {
        Path::new(param)
    }
}

/// A no cost conversion allowing any `T` to be used in place of an `Option<T>`
/// in path parts.
impl<A, T: FromUriParam<uri::Path, A>> FromUriParam<uri::Path, A> for Option<T> {
    type Target = T::Target;

    #[inline(always)]
    fn from_uri_param(param: A) -> Self::Target {
        T::from_uri_param(param)
    }
}

/// A no cost conversion allowing any `T` to be used in place of an `Result<T,
/// E>` in path parts.
impl<A, E, T: FromUriParam<uri::Path, A>> FromUriParam<uri::Path, A> for Result<T, E> {
    type Target = T::Target;

    #[inline(always)]
    fn from_uri_param(param: A) -> Self::Target {
        T::from_uri_param(param)
    }
}
