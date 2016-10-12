use error::Error;

/// Trait to create instance of some type from an HTTP form; used by code
/// generation for `form` route parameters.
///
/// This trait can be automatically derived via the
/// [rocket_codegen](/rocket_codegen) plugin:
///
/// ```rust,ignore
/// #![feature(plugin, custom_derive)]
/// #![plugin(rocket_codegen)]
///
/// extern crate rocket;
///
/// #[derive(FromForm)]
/// struct TodoTask {
///     description: String,
///     completed: bool
/// }
/// ```
///
/// When deriving `FromForm`, every field in the structure must implement
/// [FromFormValue](trait.FromFormValue.html). If you implement `FormForm`
/// yourself, use the [FormItems](struct.FormItems.html) iterator to iterate
/// through the form key/value pairs.
pub trait FromForm<'f>: Sized {
    /// The associated error which can be returned from parsing.
    type Error;

    /// Parses an instance of `Self` from a raw HTTP form
    /// (`application/x-www-form-urlencoded data`) or returns an `Error` if one
    /// cannot be parsed.
    fn from_form_string(form_string: &'f str) -> Result<Self, Self::Error>;
}

/// This implementation should only be used during debugging!
impl<'f> FromForm<'f> for &'f str {
    type Error = Error;
    fn from_form_string(s: &'f str) -> Result<Self, Error> {
        Ok(s)
    }
}

