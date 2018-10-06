#![feature(proc_macro_diagnostic, proc_macro_span)]
#![feature(crate_visibility_modifier)]
#![feature(transpose_result)]
#![feature(rustc_private)]
#![recursion_limit="128"]

//! # Rocket - Code Generation
//!
//! This crate implements the code generation portions of Rocket. This includes
//! custom derives, custom attributes, and procedural macros. The documentation
//! here is purely technical. The code generation facilities are documented
//! thoroughly in the [Rocket programming guide](https://rocket.rs/guide).
//!
//! ## **Table of Contents**
//!
//!   1. [Custom Attributes](#custom-attributes)
//!       * [`route`, `get`, `put`, ...](#route-attributes)
//!       * [`catch`](#catch-attribute)
//!   2. [Custom Derives](#custom-derives)
//!       * [`FromForm`](#fromform)
//!       * [`FromFormValue`](#fromformvalue)
//!       * [`Responder`](#responder)
//!   3. [Procedural Macros](#procedural-macros)
//!       * [`routes`, `catchers`](#routes-and-catchers)
//!       * [`uri`](#typed-uris-uri)
//!   4. [Debugging Generated Code](#debugging-codegen)
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
//! ## Custom Derives
//!
//! This crate* implements the following custom derives:
//!
//!   * **FromForm**
//!   * **FromFormValue**
//!   * **Responder**
//!
//! <small>
//!   * In reality, all of these custom derives are currently implemented by the
//!   `rocket_codegen_next` crate. Nonetheless, they are documented here.
//! </small>
//!
//! ### `FromForm`
//!
//! The [`FromForm`] derive can be applied to structures with named fields:
//!
//! ```rust,ignore
//! #[derive(FromForm)]
//! struct MyStruct {
//!     field: usize,
//!     other: String
//! }
//! ```
//!
//! Each field's type is required to implement [`FromFormValue`].
//!
//! The derive accepts one field attribute: `form`, with the following syntax:
//!
//! ```text
//! form := 'field' '=' '"' IDENT '"'
//!
//! IDENT := valid identifier, as defined by Rust
//! ```
//!
//! When applied, the attribute looks as follows:
//!
//! ```rust,ignore
//! #[derive(FromForm)]
//! struct MyStruct {
//!     field: usize,
//!     #[form(field = "renamed_field")]
//!     other: String
//! }
//! ```
//!
//! The derive generates an implementation for the [`FromForm`] trait. The
//! implementation parses a form whose field names match the field names of the
//! structure on which the derive was applied. Each field's value is parsed with
//! the [`FromFormValue`] implementation of the field's type. The `FromForm`
//! implementation succeeds only when all of the field parses succeed.
//!
//! The `form` field attribute can be used to direct that a different incoming
//! field name is expected. In this case, the `field` name in the attribute is
//! used instead of the structure's actual field name when parsing a form.
//!
//! [`FromForm`]: rocket::request::FromForm
//! [`FromFormValue`]: rocket::request::FromFormValue
//!
//! ### `FromFormValue`
//!
//! The [`FromFormValue`] derive can be applied to enums with nullary
//! (zero-length) fields:
//!
//! ```rust,ignore
//! #[derive(FromFormValue)]
//! enum MyValue {
//!     First,
//!     Second,
//!     Third,
//! }
//! ```
//!
//! The derive generates an implementation of the [`FromFormValue`] trait for
//! the decorated `enum`. The implementation returns successfully when the form
//! value matches, case insensitively, the stringified version of a variant's
//! name, returning an instance of said variant.
//!
//! As an example, for the `enum` above, the form values `"first"`, `"FIRST"`,
//! `"fiRSt"`, and so on would parse as `MyValue::First`, while `"second"` and
//! `"third"` would parse as `MyValue::Second` and `MyValue::Third`,
//! respectively.
//!
//! The `form` field attribute can be used to change the string that is compared
//! against for a given variant:
//!
//! ```rust,ignore
//! #[derive(FromFormValue)]
//! enum MyValue {
//!     First,
//!     Second,
//!     #[form(value = "fourth")]
//!     Third,
//! }
//! ```
//!
//! The attribute's grammar is:
//!
//! ```text
//! form := 'field' '=' STRING_LIT
//!
//! STRING_LIT := any valid string literal, as defined by Rust
//! ```
//!
//! The attribute accepts a single string parameter of name `value`
//! corresponding to the string to use to match against for the decorated
//! variant. In the example above, the the strings `"fourth"`, `"FOUrth"` and so
//! on would parse as `MyValue::Third`.
//!
//! ## `Responder`
//!
//! The [`Responder`] derive can be applied to enums and named structs. When
//! applied to enums, variants must have at least one field. When applied to
//! structs, the struct must have at least one field.
//!
//! ```rust,ignore
//! #[derive(Responder)]
//! enum MyResponder {
//!     A(String),
//!     B(OtherResponse, ContentType),
//! }
//!
//! #[derive(Responder)]
//! struct MyResponder {
//!     inner: OtherResponder,
//!     header: ContentType,
//! }
//! ```
//!
//! The derive generates an implementation of the [`Responder`] trait for the
//! decorated enum or structure. The derive uses the _first_ field of a variant
//! or structure to generate a `Response`. As such, the type of the first field
//! must implement [`Responder`]. The remaining fields of a variant or structure
//! are set as headers in the produced [`Response`] using
//! [`Response::set_header()`]. As such, every other field (unless explicitly
//! ignored, explained next) must implement `Into<Header>`.
//!
//! Except for the first field, fields decorated with `#[response(ignore)]` are
//! ignored by the derive:
//!
//! ```rust,ignore
//! #[derive(Responder)]
//! enum MyResponder {
//!     A(String),
//!     B(OtherResponse, ContentType, #[response(ignore)] Other),
//! }
//!
//! #[derive(Responder)]
//! struct MyResponder {
//!     inner: InnerResponder,
//!     header: ContentType,
//!     #[response(ignore)]
//!     other: Other,
//! }
//! ```
//!
//! Decorating the first field with `#[response(ignore)]` has no effect.
//!
//! Additionally, the `response` attribute can be used on named structures and
//! enum variants to override the status and/or content-type of the [`Response`]
//! produced by the generated implementation. The `response` attribute used in
//! these positions has the following grammar:
//!
//! ```text
//! response := parameter (',' parameter)?
//!
//! parameter := 'status' '=' STATUS
//!            | 'content_type' '=' CONTENT_TYPE
//!
//! STATUS := unsigned integer >= 100 and < 600
//! CONTENT_TYPE := string literal, as defined by Rust, identifying a valid
//!                 Content-Type, as defined by Rocket
//! ```
//!
//! It can be used as follows:
//!
//! ```rust,ignore
//! #[derive(Responder)]
//! enum Error {
//!     #[response(status = 500, content_type = "json")]
//!     A(String),
//!     #[response(status = 404)]
//!     B(OtherResponse, ContentType),
//! }
//!
//! #[derive(Responder)]
//! #[response(status = 400)]
//! struct MyResponder {
//!     inner: InnerResponder,
//!     header: ContentType,
//!     #[response(ignore)]
//!     other: Other,
//! }
//! ```
//!
//! The attribute accepts two key/value pairs: `status` and `content_type`. The
//! value of `status` must be an unsigned integer representing a valid status
//! code. The [`Response`] produced from the generated implementation will have
//! its status overriden to this value.
//!
//! The value of `content_type` must be a valid media-type in `top/sub` form or
//! `shorthand` form. Examples include:
//!
//!   * `"text/html"`
//!   * `"application/x-custom"`
//!   * `"html"`
//!   * `"json"`
//!   * `"plain"`
//!   * `"binary"`
//!
//! The [`Response`] produced from the generated implementation will have its
//! content-type overriden to this value.
//!
//! [`Responder`]: rocket::response::Responder
//! [`Response`]: rocket::Response
//! [`Response::set_header()`]: rocket::Response::set_header()
//!
//! ## Procedural Macros
//!
//! This crate implements the following procedural macros:
//!
//!   * **routes**
//!   * **catchers**
//!   * **uri**
//!
//! ## Routes and Catchers
//!
//! The syntax for `routes!` and `catchers!` is defined as:
//!
//! ```text
//! macro := PATH (',' PATH)*
//!
//! PATH := a path, as defined by Rust
//! ```
//!
//! ### Typed URIs: `uri!`
//!
//! The `uri!` macro creates a type-safe URI given a route and values for the
//! route's URI parameters. The inputs to the macro are the path to a route, a
//! colon, and one argument for each dynamic parameter (parameters in `<>`) in
//! the route's path.
//!
//! For example, for the following route:
//!
//! ```rust,ignore
//! #[get("/person/<name>/<age>")]
//! fn person(name: String, age: u8) -> String {
//!     format!("Hello {}! You're {} years old.", name, age)
//! }
//! ```
//!
//! A URI can be created as follows:
//!
//! ```rust,ignore
//! // with unnamed parameters, in route path declaration order
//! let mike = uri!(person: "Mike", 28);
//!
//! // with named parameters, order irrelevant
//! let mike = uri!(person: name = "Mike", age = 28);
//! let mike = uri!(person: age = 28, name = "Mike");
//!
//! // with a specific mount-point
//! let mike = uri!("/api", person: name = "Mike", age = 28);
//! ```
//!
//! #### Grammar
//!
//! The grammar for the `uri!` macro is as follows:
//!
//! ```text
//! uri := (mount ',')? PATH (':' params)?
//!
//! mount = STRING
//! params := unnamed | named
//! unnamed := EXPR (',' EXPR)*
//! named := IDENT = EXPR (',' named)?
//!
//! EXPR := a valid Rust expression (examples: `foo()`, `12`, `"hey"`)
//! IDENT := a valid Rust identifier (examples: `name`, `age`)
//! STRING := an uncooked string literal, as defined by Rust (example: `"hi"`)
//! PATH := a path, as defined by Rust (examples: `route`, `my_mod::route`)
//! ```
//!
//! #### Semantics
//!
//! The `uri!` macro returns an [`Origin`](rocket::uri::Origin) structure with
//! the URI of the supplied route interpolated with the given values. Note that
//! `Origin` implements `Into<Uri>` (and by extension, `TryInto<Uri>`), so it
//! can be converted into a [`Uri`](rocket::uri::Uri) using `.into()` as needed.
//!
//!
//! A `uri!` invocation only typechecks if the type of every value in the
//! invocation matches the type declared for the parameter in the given route.
//! The [`FromUriParam`] trait is used to typecheck and perform a conversion for
//! each value. If a `FromUriParam<S>` implementation exists for a type `T`,
//! then a value of type `S` can be used in `uri!` macro for a route URI
//! parameter declared with a type of `T`. For example, the following
//! implementation, provided by Rocket, allows an `&str` to be used in a `uri!`
//! invocation for route URI parameters declared as `String`:
//!
//! ```rust,ignore
//! impl<'a> FromUriParam<&'a str> for String
//! ```
//!
//! Each value passed into `uri!` is rendered in its appropriate place in the
//! URI using the [`UriDisplay`] implementation for the value's type. The
//! `UriDisplay` implementation ensures that the rendered value is URI-safe.
//!
//! If a mount-point is provided, the mount-point is prepended to the route's
//! URI.
//!
//! [`Uri`]: http::uri::URI
//! [`FromUriParam`]: http::uri::FromUriParam
//! [`UriDisplay`]: http::uri::UriDisplay
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

#[proc_macro_derive(FromFormValue, attributes(form))]
pub fn derive_from_form_value(input: TokenStream) -> TokenStream {
    emit!(derive::from_form_value::derive_from_form_value(input))
}

#[proc_macro_derive(FromForm, attributes(form))]
pub fn derive_from_form(input: TokenStream) -> TokenStream {
    emit!(derive::from_form::derive_from_form(input))
}

#[proc_macro_derive(Responder, attributes(response))]
pub fn derive_responder(input: TokenStream) -> TokenStream {
    emit!(derive::responder::derive_responder(input))
}

#[proc_macro_attribute]
pub fn catch(args: TokenStream, input: TokenStream) -> TokenStream {
    emit!(attribute::catch::catch_attribute(args, input))
}

#[proc_macro]
pub fn routes(input: TokenStream) -> TokenStream {
    emit!(bang::routes_macro(input))
}

#[proc_macro]
pub fn catchers(input: TokenStream) -> TokenStream {
    emit!(bang::catchers_macro(input))
}

#[proc_macro]
pub fn uri(input: TokenStream) -> TokenStream {
    emit!(bang::uri_macro(input))
}

#[doc(hidden)]
#[proc_macro]
pub fn rocket_internal_uri(input: TokenStream) -> TokenStream {
    emit!(bang::uri_internal_macro(input))
}
