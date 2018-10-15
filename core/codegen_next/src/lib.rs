#![feature(proc_macro_diagnostic, proc_macro_span)]
#![feature(crate_visibility_modifier)]
#![feature(transpose_result)]
#![feature(rustc_private)]
#![recursion_limit="128"]

#![doc(html_root_url = "https://api.rocket.rs/0.4.0-dev")]
#![doc(html_favicon_url = "https://rocket.rs/favicon.ico")]
#![doc(html_logo_url = "https://rocket.rs/images/logo-boxed.png")]

//! # Rocket - Code Generation
//!
//! This crate implements the code generation portions of Rocket. This includes
//! custom derives, custom attributes, and procedural macros. The documentation
//! here is purely technical. The code generation facilities are documented
//! thoroughly in the [Rocket programming guide](https://rocket.rs/guide).
//!
//! ## Custom Attributes
//!
//! This crate implements the following custom attributes:
//!
//!   * **route**
//!   * **get**
//!   * **put**
//!   * **post**
//!   * **delete**
//!   * **head**
//!   * **patch**
//!   * **options**
//!   * **catch**
//!
//! ### Route Attributes
//!
//! The grammar for all _route_ attributes, including **route**, **get**,
//! **put**, **post**, **delete**, **head**, **patch**, and **options** is
//! defined as:
//!
//! ```text
//! route := METHOD? '(' ('path' '=')? path (',' kv_param)* ')'
//!
//! path := URI_SEG
//!       | DYNAMIC_PARAM
//!       | '?' DYNAMIC_PARAM
//!       | path '/' path
//!       (string literal)
//!
//! kv_param := 'rank' '=' INTEGER
//!           | 'format' '=' STRING
//!           | 'data' '=' DYNAMIC_PARAM
//!
//! INTEGER := isize, as defined by Rust
//! STRING := UTF-8 string literal, as defined by Rust
//! IDENT := valid identifier, as defined by Rust
//!
//! URI_SEG := valid HTTP URI Segment
//! DYNAMIC_PARAM := '<' IDENT '..'? '>' (string literal)
//! ```
//!
//! Note that the **route** attribute takes a method as its first argument,
//! while the remaining do not. That is, **route** looks like:
//!
//! ```rust,ignore
//! #[route(GET, path = "/hello")]
//! ```
//!
//! while the equivalent using **get** looks like:
//!
//! ```rust,ignore
//! #[get("/hello")]
//! ```
//!
//! ### Catch Attribute
//!
//! The syntax for the **catch** attribute is:
//!
//! ```text
//! catch := INTEGER
//! ```
//!
//! A use of the `catch` attribute looks like:
//!
//! ```rust,ignore
//! #[catch(404)]
//! ```
//!
//! # Debugging Codegen
//!
//! When the `ROCKET_CODEGEN_DEBUG` environment variable is set, this crate logs
//! the items it has generated to the console at compile-time. For example, you
//! might run the following to build a Rocket application with codegen logging
//! enabled:
//!
//! ```sh
//! ROCKET_CODEGEN_DEBUG=1 cargo build
//! ```

#[macro_use] extern crate quote;
#[macro_use] extern crate derive_utils;
extern crate proc_macro;
extern crate rocket_http as http;
extern crate indexmap;

extern crate syntax_pos;

#[macro_use] mod proc_macro_ext;
mod derive;
mod attribute;
mod bang;
mod http_codegen;
mod syn_ext;

use http::Method;
use proc_macro::TokenStream;
crate use derive_utils::proc_macro2;

crate static ROUTE_STRUCT_PREFIX: &str = "static_rocket_route_info_for_";
crate static CATCH_STRUCT_PREFIX: &str = "static_rocket_catch_info_for_";
crate static CATCH_FN_PREFIX: &str = "rocket_catch_fn_";
crate static ROUTE_FN_PREFIX: &str = "rocket_route_fn_";
crate static URI_MACRO_PREFIX: &str = "rocket_uri_macro_";
crate static ROCKET_PARAM_PREFIX: &str = "__rocket_param_";

macro_rules! emit {
    ($tokens:expr) => ({
        let tokens = $tokens;
        if ::std::env::var_os("ROCKET_CODEGEN_DEBUG").is_some() {
            ::proc_macro::Span::call_site()
                .note("emitting Rocket code generation debug output")
                .note(tokens.to_string())
                .emit()
        }

        tokens
    })
}

macro_rules! route_attribute {
    ($name:ident => $method:expr) => (
        #[proc_macro_attribute]
        pub fn $name(args: TokenStream, input: TokenStream) -> TokenStream {
            emit!(attribute::route::route_attribute($method, args, input))
        }
    )
}

route_attribute!(route => None);
route_attribute!(get => Method::Get);
route_attribute!(put => Method::Put);
route_attribute!(post => Method::Post);
route_attribute!(delete => Method::Delete);
route_attribute!(head => Method::Head);
route_attribute!(patch => Method::Patch);
route_attribute!(options => Method::Options);

/// Derive for the [`FromFormValue`] trait.
///
/// The [`FromFormValue`] derive can be applied to enums with nullary
/// (zero-length) fields:
///
/// ```rust
/// # #[macro_use] extern crate rocket;
/// #
/// #[derive(FromFormValue)]
/// enum MyValue {
///     First,
///     Second,
///     Third,
/// }
/// ```
///
/// The derive generates an implementation of the [`FromFormValue`] trait for
/// the decorated `enum`. The implementation returns successfully when the form
/// value matches, case insensitively, the stringified version of a variant's
/// name, returning an instance of said variant. If there is no match, an error
/// ([`FromFormValue::Error`]) of type [`&RawStr`] is returned, the value of
/// which is the raw form field value that failed to match.
///
/// As an example, for the `enum` above, the form values `"first"`, `"FIRST"`,
/// `"fiRSt"`, and so on would parse as `MyValue::First`, while `"second"` and
/// `"third"` would parse as `MyValue::Second` and `MyValue::Third`,
/// respectively.
///
/// The `form` field attribute can be used to change the string that is compared
/// against for a given variant:
///
/// ```rust
/// # #[macro_use] extern crate rocket;
/// #
/// #[derive(FromFormValue)]
/// enum MyValue {
///     First,
///     Second,
///     #[form(value = "fourth")]
///     Third,
/// }
/// ```
///
/// The `#[form]` attribute's grammar is:
///
/// ```text
/// form := 'field' '=' STRING_LIT
///
/// STRING_LIT := any valid string literal, as defined by Rust
/// ```
///
/// The attribute accepts a single string parameter of name `value`
/// corresponding to the string to use to match against for the decorated
/// variant. In the example above, the the strings `"fourth"`, `"FOUrth"` and so
/// on would parse as `MyValue::Third`.
///
/// [`FromFormValue`]: ../rocket/request/trait.FromFormValue.html
/// [`FromFormValue::Error`]: ../rocket/request/trait.FromFormValue.html#associatedtype.Error
/// [`&RawStr`]: ../rocket/http/struct.RawStr.html
// FIXME(rustdoc): We should be able to refer to items in `rocket`.
#[proc_macro_derive(FromFormValue, attributes(form))]
pub fn derive_from_form_value(input: TokenStream) -> TokenStream {
    emit!(derive::from_form_value::derive_from_form_value(input))
}

/// Derive for the [`FromForm`] trait.
///
/// The [`FromForm`] derive can be applied to structures with named fields:
///
/// ```rust
/// # #[macro_use] extern crate rocket;
/// #
/// #[derive(FromForm)]
/// struct MyStruct {
///     field: usize,
///     other: String
/// }
/// ```
///
/// Each field's type is required to implement [`FromFormValue`].
///
/// The derive generates an implementation of the [`FromForm`] trait. The
/// implementation parses a form whose field names match the field names of the
/// structure on which the derive was applied. Each field's value is parsed with
/// the [`FromFormValue`] implementation of the field's type. The `FromForm`
/// implementation succeeds only when all of the field parses succeed. If
/// parsing fails, an error ([`FromForm::Error`]) of type [`FormParseError`] is
/// returned.
///
/// The derive accepts one field attribute: `form`, with the following syntax:
///
/// ```text
/// form := 'field' '=' '"' IDENT '"'
///
/// IDENT := valid identifier, as defined by Rust
/// ```
///
/// When applied, the attribute looks as follows:
///
/// ```rust
/// # #[macro_use] extern crate rocket;
/// #
/// #[derive(FromForm)]
/// struct MyStruct {
///     field: usize,
///     #[form(field = "renamed_field")]
///     other: String
/// }
/// ```
///
/// The field attribute directs that a different incoming field name is
/// expected, and the value of the `field` attribute is used instead of the
/// structure's actual field name when parsing a form. In the example above, the
/// value of the `MyStruct::other` struct field will be parsed from the incoming
/// form's `renamed_field` field.
///
/// [`FromForm`]: ../rocket/request/trait.FromForm.html
/// [`FromFormValue`]: ../rocket/request/trait.FromFormValue.html
/// [`FormParseError`]: ../rocket/request/enum.FormParseError.html
/// [`FromForm::Error`]: ../rocket/request/trait.FromForm.html#associatedtype.Error
#[proc_macro_derive(FromForm, attributes(form))]
pub fn derive_from_form(input: TokenStream) -> TokenStream {
    emit!(derive::from_form::derive_from_form(input))
}

/// Derive for the [`Responder`] trait.
///
/// The [`Responder`] derive can be applied to enums and structs with named
/// fields. When applied to enums, variants must have at least one field. When
/// applied to structs, the struct must have at least one field.
///
/// ```rust
/// # #[macro_use] extern crate rocket;
/// # use std::fs::File;
/// # use rocket::http::ContentType;
/// # type OtherResponder = MyResponderA;
/// #
/// #[derive(Responder)]
/// enum MyResponderA {
///     A(String),
///     B(File, ContentType),
/// }
///
/// #[derive(Responder)]
/// struct MyResponderB {
///     inner: OtherResponder,
///     header: ContentType,
/// }
/// ```
///
/// The derive generates an implementation of the [`Responder`] trait for the
/// decorated enum or structure. The derive uses the _first_ field of a variant
/// or structure to generate a [`Response`]. As such, the type of the first
/// field must implement [`Responder`]. The remaining fields of a variant or
/// structure are set as headers in the produced [`Response`] using
/// [`Response::set_header()`]. As such, every other field (unless explicitly
/// ignored, explained next) must implement `Into<Header>`.
///
/// Except for the first field, fields decorated with `#[response(ignore)]` are
/// ignored by the derive:
///
/// ```rust
/// # #[macro_use] extern crate rocket;
/// # use std::fs::File;
/// # use rocket::http::ContentType;
/// # use rocket::response::NamedFile;
/// # type Other = usize;
/// #
/// #[derive(Responder)]
/// enum MyResponder {
///     A(String),
///     B(File, ContentType, #[response(ignore)] Other),
/// }
///
/// #[derive(Responder)]
/// struct MyOtherResponder {
///     inner: NamedFile,
///     header: ContentType,
///     #[response(ignore)]
///     other: Other,
/// }
/// ```
///
/// Decorating the first field with `#[response(ignore)]` has no effect.
///
/// Additionally, the `response` attribute can be used on named structures and
/// enum variants to override the status and/or content-type of the [`Response`]
/// produced by the generated implementation. The `response` attribute used in
/// these positions has the following grammar:
///
/// ```text
/// response := parameter (',' parameter)?
///
/// parameter := 'status' '=' STATUS
///            | 'content_type' '=' CONTENT_TYPE
///
/// STATUS := unsigned integer >= 100 and < 600
/// CONTENT_TYPE := string literal, as defined by Rust, identifying a valid
///                 Content-Type, as defined by Rocket
/// ```
///
/// It can be used as follows:
///
/// ```rust
/// # #[macro_use] extern crate rocket;
/// # use rocket::http::ContentType;
/// # use rocket::response::NamedFile;
/// # type Other = usize;
/// # type InnerResponder = String;
/// #
/// #[derive(Responder)]
/// enum Error {
///     #[response(status = 500, content_type = "json")]
///     A(String),
///     #[response(status = 404)]
///     B(NamedFile, ContentType),
/// }
///
/// #[derive(Responder)]
/// #[response(status = 400)]
/// struct MyResponder {
///     inner: InnerResponder,
///     header: ContentType,
///     #[response(ignore)]
///     other: Other,
/// }
/// ```
///
/// The attribute accepts two key/value pairs: `status` and `content_type`. The
/// value of `status` must be an unsigned integer representing a valid status
/// code. The [`Response`] produced from the generated implementation will have
/// its status overriden to this value.
///
/// The value of `content_type` must be a valid media-type in `top/sub` form or
/// `shorthand` form. Examples include:
///
///   * `"text/html"`
///   * `"application/x-custom"`
///   * `"html"`
///   * `"json"`
///   * `"plain"`
///   * `"binary"`
///
/// See [`ContentType::parse_flexible()`] for a full list of available
/// shorthands. The [`Response`] produced from the generated implementation will
/// have its content-type overriden to this value.
///
/// [`Responder`]: ../rocket/response/trait.Responder.html
/// [`Response`]: ../rocket/struct.Response.html
/// [`Response::set_header()`]: ../rocket/response/struct.Response.html#method.set_header
/// [`ContentType::parse_flexible()`]: ../rocket/http/struct.ContentType.html#method.parse_flexible
#[proc_macro_derive(Responder, attributes(response))]
pub fn derive_responder(input: TokenStream) -> TokenStream {
    emit!(derive::responder::derive_responder(input))
}

#[proc_macro_attribute]
pub fn catch(args: TokenStream, input: TokenStream) -> TokenStream {
    emit!(attribute::catch::catch_attribute(args, input))
}

/// Generates a [`Vec`] of [`Route`]s from a set of route paths.
///
/// The `routes!` macro expands a list of route paths into a [`Vec`] of their
/// corresponding [`Route`] structures. For example, given the following routes:
///
/// ```rust
/// # #![feature(proc_macro_hygiene, decl_macro)]
/// # #[macro_use] extern crate rocket;
/// #
/// #[get("/")]
/// fn index() { /* .. */ }
///
/// mod person {
///     #[post("/hi/<person>")]
///     pub fn hello(person: String) { /* .. */ }
/// }
/// ```
///
/// The `routes!` macro can be used as:
///
/// ```rust
/// # #![feature(proc_macro_hygiene, decl_macro)]
/// # #[macro_use] extern crate rocket;
/// #
/// # use rocket::http::Method;
/// #
/// # #[get("/")] fn index() { /* .. */ }
/// # mod person {
/// #   #[post("/hi/<person>")] pub fn hello(person: String) { /* .. */ }
/// # }
/// let my_routes = routes![index, person::hello];
/// assert_eq!(my_routes.len(), 2);
///
/// let index_route = &my_routes[0];
/// assert_eq!(index_route.method, Method::Get);
/// assert_eq!(index_route.name, Some("index"));
/// assert_eq!(index_route.uri.path(), "/");
///
/// let hello_route = &my_routes[1];
/// assert_eq!(hello_route.method, Method::Post);
/// assert_eq!(hello_route.name, Some("hello"));
/// assert_eq!(hello_route.uri.path(), "/hi/<person>");
/// ```
///
/// The grammar for `routes!` is defined as:
///
/// ```text
/// routes := PATH (',' PATH)*
///
/// PATH := a path, as defined by Rust
/// ```
///
/// [`Route`]: ../rocket/struct.Route.html
#[proc_macro]
pub fn routes(input: TokenStream) -> TokenStream {
    emit!(bang::routes_macro(input))
}

/// Generates a [`Vec`] of [`Catcher`]s from a set of catcher paths.
///
/// The `catchers!` macro expands a list of catcher paths into a [`Vec`] of
/// their corresponding [`Catcher`] structures. For example, given the following
/// catchers:
///
/// ```rust
/// # #![feature(proc_macro_hygiene, decl_macro)]
/// # #[macro_use] extern crate rocket;
/// #
/// #[catch(404)]
/// fn not_found() { /* .. */ }
///
/// mod inner {
///     #[catch(400)]
///     pub fn unauthorized() { /* .. */ }
/// }
/// ```
///
/// The `catchers!` macro can be used as:
///
/// ```rust
/// # #![feature(proc_macro_hygiene, decl_macro)]
/// # #[macro_use] extern crate rocket;
/// #
/// # #[catch(404)] fn not_found() { /* .. */ }
/// # mod inner {
/// #     #[catch(401)] pub fn unauthorized() { /* .. */ }
/// # }
/// #
/// let my_catchers = catchers![not_found, inner::unauthorized];
/// assert_eq!(my_catchers.len(), 2);
///
/// let not_found = &my_catchers[0];
/// assert_eq!(not_found.code, 404);
///
/// let unauthorized = &my_catchers[1];
/// assert_eq!(unauthorized.code, 401);
/// ```
///
/// The grammar for `catchers!` is defined as:
///
/// ```text
/// catchers := PATH (',' PATH)*
///
/// PATH := a path, as defined by Rust
/// ```
///
/// [`Catcher`]: ../rocket/struct.Catcher.html
#[proc_macro]
pub fn catchers(input: TokenStream) -> TokenStream {
    emit!(bang::catchers_macro(input))
}

/// Type safe generation of route URIs.
///
/// The `uri!` macro creates a type-safe, URL safe URI given a route and values
/// for the route's URI parameters. The inputs to the macro are the path to a
/// route, a colon, and one argument for each dynamic parameter (parameters in
/// `<>`) in the route's path and query.
///
/// For example, for the following route:
///
/// ```rust
/// # #![feature(proc_macro_hygiene, decl_macro)]
/// # #[macro_use] extern crate rocket;
/// #
/// #[get("/person/<name>/<age>")]
/// fn person(name: String, age: u8) -> String {
///     format!("Hello {}! You're {} years old.", name, age)
/// }
/// ```
///
/// A URI can be created as follows:
///
/// ```rust
/// # #![feature(proc_macro_hygiene, decl_macro)]
/// # #[macro_use] extern crate rocket;
/// #
/// # #[get("/person/<name>/<age>")]
/// # fn person(name: String, age: u8) { }
/// #
/// // with unnamed parameters, in route path declaration order
/// let mike = uri!(person: "Mike Smith", 28);
/// assert_eq!(mike.path(), "/person/Mike%20Smith/28");
///
/// // with named parameters, order irrelevant
/// let mike = uri!(person: name = "Mike", age = 28);
/// let mike = uri!(person: age = 28, name = "Mike");
/// assert_eq!(mike.path(), "/person/Mike/28");
///
/// // with a specific mount-point
/// let mike = uri!("/api", person: name = "Mike", age = 28);
/// assert_eq!(mike.path(), "/api/person/Mike/28");
/// ```
///
/// ## Grammar
///
/// The grammar for the `uri!` macro is:
///
/// ```text
/// uri := (mount ',')? PATH (':' params)?
///
/// mount = STRING
/// params := unnamed | named
/// unnamed := EXPR (',' EXPR)*
/// named := IDENT = EXPR (',' named)?
///
/// EXPR := a valid Rust expression (examples: `foo()`, `12`, `"hey"`)
/// IDENT := a valid Rust identifier (examples: `name`, `age`)
/// STRING := an uncooked string literal, as defined by Rust (example: `"hi"`)
/// PATH := a path, as defined by Rust (examples: `route`, `my_mod::route`)
/// ```
///
/// ## Semantics
///
/// The `uri!` macro returns an [`Origin`] structure with the URI of the
/// supplied route interpolated with the given values. Note that `Origin`
/// implements `Into<Uri>` (and by extension, `TryInto<Uri>`), so it can be
/// converted into a [`Uri`] using `.into()` as needed.
///
/// A `uri!` invocation only typechecks if the type of every value in the
/// invocation matches the type declared for the parameter in the given route.
/// The [`FromUriParam`] trait is used to typecheck and perform a conversion for
/// each value. If a `FromUriParam<S>` implementation exists for a type `T`,
/// then a value of type `S` can be used in `uri!` macro for a route URI
/// parameter declared with a type of `T`. For example, the following
/// implementation, provided by Rocket, allows an `&str` to be used in a `uri!`
/// invocation for route URI parameters declared as `String`:
///
/// ```rust,ignore
/// impl<'a> FromUriParam<&'a str> for String { .. }
/// ```
///
/// Each value passed into `uri!` is rendered in its appropriate place in the
/// URI using the [`UriDisplay`] implementation for the value's type. The
/// `UriDisplay` implementation ensures that the rendered value is URI-safe.
///
/// If a mount-point is provided, the mount-point is prepended to the route's
/// URI.
///
/// [`Uri`]: ../rocket/http/uri/enum.Uri.html
/// [`Origin`]: ../rocket/http/uri/struct.Origin.html
/// [`FromUriParam`]: ../rocket/http/uri/trait.FromUriParam.html
/// [`UriDisplay`]: ../rocket/http/uri/trait.UriDisplay.html
#[proc_macro]
pub fn uri(input: TokenStream) -> TokenStream {
    emit!(bang::uri_macro(input))
}

#[doc(hidden)]
#[proc_macro]
pub fn rocket_internal_uri(input: TokenStream) -> TokenStream {
    emit!(bang::uri_internal_macro(input))
}
