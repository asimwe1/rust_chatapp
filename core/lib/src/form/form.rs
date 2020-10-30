use std::ops::{Deref, DerefMut};

use crate::request::Request;
use crate::data::{Data, FromData, Outcome};
use crate::http::{RawStr, ext::IntoOwned};
use crate::form::parser::{Parser, RawStrParser, Buffer};
use crate::form::prelude::*;

/// A data guard for [`FromForm`] types.
///
/// This type implements the [`FromData`] trait. It provides a generic means to
/// parse arbitrary structures from incoming form data of any kind.
///
/// See the [forms guide](https://rocket.rs/master/guide/requests#forms) for
/// general form support documentation.
///
/// # Leniency
///
/// A `Form<T>` will parse successfully from an incoming form if the form
/// contains a superset of the fields in `T`. Said another way, a `Form<T>`
/// automatically discards extra fields without error. For instance, if an
/// incoming form contains the fields "a", "b", and "c" while `T` only contains
/// "a" and "c", the form _will_ parse as `Form<T>`. To parse strictly, use the
/// [`Strict`](crate::form::Strict) form guard.
///
/// # Usage
///
/// This type can be used with any type that implements the `FromForm` trait.
/// The trait can be automatically derived; see the [`FromForm`] documentation
/// for more information on deriving or implementing the trait.
///
/// Because `Form` implements `FromData`, it can be used directly as a target of
/// the `data = "<param>"` route parameter as long as its generic type
/// implements the `FromForm` trait:
///
/// ```rust
/// # #[macro_use] extern crate rocket;
/// use rocket::form::Form;
/// use rocket::http::RawStr;
///
/// #[derive(FromForm)]
/// struct UserInput<'r> {
///     value: &'r str
/// }
///
/// #[post("/submit", data = "<user_input>")]
/// fn submit_task(user_input: Form<UserInput<'_>>) -> String {
///     format!("Your value: {}", user_input.value)
/// }
/// ```
///
/// A type of `Form<T>` automatically dereferences into an `&T` or `&mut T`,
/// though you can also transform a `Form<T>` into a `T` by calling
/// [`into_inner()`](Form::into_inner()). Thanks to automatic dereferencing, you
/// can access fields of `T` transparently through a `Form<T>`, as seen above
/// with `user_input.value`.
///
/// ## Data Limits
///
/// The default size limit for incoming form data is 32KiB. Setting a limit
/// protects your application from denial of service (DOS) attacks and from
/// resource exhaustion through high memory consumption. The limit can be
/// modified by setting the `limits.form` configuration parameter. For instance,
/// to increase the forms limit to 512KiB for all environments, you may add the
/// following to your `Rocket.toml`:
///
/// ```toml
/// [global.limits]
/// form = 524288
/// ```
///
/// See the [`Limits`](crate::data::Limits) docs for more.
#[derive(Debug)]
pub struct Form<T>(T);

impl<T> Form<T> {
    /// Consumes `self` and returns the inner value.
    ///
    /// Note that since `Form` implements [`Deref`] and [`DerefMut`] with
    /// target `T`, reading and writing an inner value can be accomplished
    /// transparently.
    ///
    /// # Example
    ///
    /// ```rust
    /// # #[macro_use] extern crate rocket;
    /// use rocket::form::Form;
    ///
    /// #[derive(FromForm)]
    /// struct MyForm {
    ///     field: String,
    /// }
    ///
    /// #[post("/submit", data = "<form>")]
    /// fn submit(form: Form<MyForm>) -> String {
    ///     // We can read or mutate a value transparently:
    ///     let field: &str = &form.field;
    ///
    ///     // To gain ownership, however, use `into_inner()`:
    ///     form.into_inner().field
    /// }
    /// ```
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> From<T> for Form<T> {
    #[inline]
    fn from(val: T) -> Form<T> {
        Form(val)
    }
}

impl Form<()> {
    /// `string` must represent a decoded string.
    pub fn values(string: &str) -> impl Iterator<Item = ValueField<'_>> {
        // WHATWG URL Living Standard 5.1 steps 1, 2, 3.1 - 3.3.
        string.split('&')
            .filter(|s| !s.is_empty())
            .map(ValueField::parse)
    }
}

impl<'r, T: FromForm<'r>> Form<T> {
    /// `string` must represent a decoded string.
    pub fn parse(string: &'r str) -> Result<'r, T> {
        // WHATWG URL Living Standard 5.1 steps 1, 2, 3.1 - 3.3.
        let mut ctxt = T::init(Options::Lenient);
        Form::values(string).for_each(|f| T::push_value(&mut ctxt, f));
        T::finalize(ctxt)
    }
}

impl<T: for<'a> FromForm<'a> + 'static> Form<T> {
    /// `string` must represent an undecoded string.
    pub fn parse_encoded(string: &RawStr) -> Result<'static, T> {
        let buffer = Buffer::new();
        let mut ctxt = T::init(Options::Lenient);
        for field in RawStrParser::new(&buffer, string) {
            T::push_value(&mut ctxt, field)
        }

        T::finalize(ctxt).map_err(|e| e.into_owned())
    }
}

impl<T> Deref for Form<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Form<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[crate::async_trait]
impl<'r, T: FromForm<'r>> FromData<'r> for Form<T> {
    type Error = Errors<'r>;

    async fn from_data(req: &'r Request<'_>, data: Data) -> Outcome<Self, Self::Error> {
        use either::Either;

        let mut parser = try_outcome!(Parser::new(req, data).await);
        let mut context = T::init(Options::Lenient);
        while let Some(field) = parser.next().await {
            match field {
                Ok(Either::Left(value)) => T::push_value(&mut context, value),
                Ok(Either::Right(data)) => T::push_data(&mut context, data).await,
                Err(e) => T::push_error(&mut context, e),
            }
        }

        match T::finalize(context) {
            Ok(value) => Outcome::Success(Form(value)),
            Err(e) => Outcome::Failure((e.status(), e)),
        }
    }
}
