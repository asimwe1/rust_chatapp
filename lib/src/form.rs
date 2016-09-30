//! Types and traits to handle form processing.
//!
//! In general, you will deal with forms in Rocket via the `form` parameter in
//! routes:
//!
//! ```rust,ignore
//! #[post("/", form = <my_form>)]
//! fn form_submit(my_form: MyType) -> ...
//! ```
//!
//! Form parameter types must implement the [FromForm](trait.FromForm.html)
//! trait, which is automatically derivable. Automatically deriving `FromForm`
//! for a structure requires that all of its fields implement
//! [FromFormValue](trait.FormFormValue.html). See the
//! [codegen](/rocket_codegen/) documentation or the [forms guide](/guide/forms)
//! for more information on forms and on deriving `FromForm`.

use url;
use error::Error;
use std::str::FromStr;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddrV4, SocketAddrV6, SocketAddr};

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

/// Trait to create instance of some type from a form value; expected from field
/// types in structs deriving `FromForm`.
///
/// # Examples
///
/// This trait is generally implemented when verifying form inputs. For example,
/// if you'd like to verify that some user is over some age in a form, then you
/// might define a new type and implement `FromFormValue` as follows:
///
/// ```rust
/// use rocket::form::FromFormValue;
/// use rocket::Error;
///
/// struct AdultAge(usize);
///
/// impl<'v> FromFormValue<'v> for AdultAge {
///     type Error = &'v str;
///
///     fn from_form_value(form_value: &'v str) -> Result<AdultAge, &'v str> {
///         match usize::from_form_value(form_value) {
///             Ok(age) if age >= 21 => Ok(AdultAge(age)),
///             _ => Err(form_value),
///         }
///     }
/// }
/// ```
///
/// This type can then be used in a `FromForm` struct as follows:
///
/// ```rust,ignore
/// #[derive(FromForm)]
/// struct User {
///     age: AdultAge,
///     ...
/// }
/// ```
pub trait FromFormValue<'v>: Sized {
    /// The associated error which can be returned from parsing. It is a good
    /// idea to have the return type be or contain an `&'v str` so that the
    /// unparseable string can be examined after a bad parse.
    type Error;

    /// Parses an instance of `Self` from an HTTP form field value or returns an
    /// `Error` if one cannot be parsed.
    fn from_form_value(form_value: &'v str) -> Result<Self, Self::Error>;

    /// Returns a default value to be used when the form field does not exist.
    /// If this returns None, then the field is required. Otherwise, this should
    /// return Some(default_value).
    fn default() -> Option<Self> {
        None
    }
}

impl<'v> FromFormValue<'v> for &'v str {
    type Error = Error;

    // This just gives the raw string.
    fn from_form_value(v: &'v str) -> Result<Self, Self::Error> {
        Ok(v)
    }
}

impl<'v> FromFormValue<'v> for String {
    type Error = &'v str;

    // This actually parses the value according to the standard.
    fn from_form_value(v: &'v str) -> Result<Self, Self::Error> {
        let decoder = url::percent_encoding::percent_decode(v.as_bytes());
        let res = decoder.decode_utf8().map_err(|_| v).map(|s| s.into_owned());
        match res {
            e@Err(_) => e,
            Ok(mut string) => Ok({
                unsafe {
                    for c in string.as_mut_vec() {
                        if *c == b'+' {
                            *c = b' ';
                        }
                    }
                }

                string
            })
        }
    }
}

impl<'v> FromFormValue<'v> for bool {
    type Error = &'v str;

    fn from_form_value(v: &'v str) -> Result<Self, Self::Error> {
        match v {
            "on" | "true" => Ok(true),
            "off" | "false" => Ok(false),
            _ => Err(v),
        }
    }
}

macro_rules! impl_with_fromstr {
    ($($T:ident),+) => ($(
        impl<'v> FromFormValue<'v> for $T {
            type Error = &'v str;
            fn from_form_value(v: &'v str) -> Result<Self, Self::Error> {
                $T::from_str(v).map_err(|_| v)
            }
        }
    )+)
}

impl_with_fromstr!(f32, f64, isize, i8, i16, i32, i64, usize, u8, u16, u32, u64,
    IpAddr, Ipv4Addr, Ipv6Addr, SocketAddrV4, SocketAddrV6, SocketAddr);

impl<'v, T: FromFormValue<'v>> FromFormValue<'v> for Option<T> {
    type Error = Error;

    fn from_form_value(v: &'v str) -> Result<Self, Self::Error> {
        match T::from_form_value(v) {
            Ok(v) => Ok(Some(v)),
            Err(_) => Ok(None),
        }
    }

    fn default() -> Option<Option<T>> {
        Some(None)
    }
}

// TODO: Add more useful implementations (range, regex, etc.).
impl<'v, T: FromFormValue<'v>> FromFormValue<'v> for Result<T, T::Error> {
    type Error = Error;

    fn from_form_value(v: &'v str) -> Result<Self, Self::Error> {
        match T::from_form_value(v) {
            ok@Ok(_) => Ok(ok),
            e@Err(_) => Ok(e),
        }
    }
}

/// Iterator over the key/value pairs of a given HTTP form string. You'll likely
/// want to use this if you're implementing [FromForm](trait.FromForm.html)
/// manually, for whatever reason, by iterating over the items in `form_string`.
///
/// # Examples
///
/// `FormItems` can be used directly as an iterator:
///
/// ```rust
/// use rocket::form::FormItems;
///
/// // prints "greeting = hello" then "username = jake"
/// let form_string = "greeting=hello&username=jake";
/// for (key, value) in FormItems(form_string) {
///     println!("{} = {}", key, value);
/// }
/// ```
///
/// This is the same example as above, but the iterator is used explicitly.
///
/// ```rust
/// use rocket::form::FormItems;
///
/// let form_string = "greeting=hello&username=jake";
/// let mut items = FormItems(form_string);
/// assert_eq!(items.next(), Some(("greeting", "hello")));
/// assert_eq!(items.next(), Some(("username", "jake")));
/// assert_eq!(items.next(), None);
/// ```
pub struct FormItems<'f>(pub &'f str);

impl<'f> Iterator for FormItems<'f> {
    type Item = (&'f str, &'f str);

    fn next(&mut self) -> Option<Self::Item> {
        let string = self.0;
        let (key, rest) = match string.find('=') {
            Some(index) => (&string[..index], &string[(index + 1)..]),
            None => return None,
        };

        let (value, remainder) = match rest.find('&') {
            Some(index) => (&rest[..index], &rest[(index + 1)..]),
            None => (rest, ""),
        };

        self.0 = remainder;
        Some((key, value))
    }
}

#[cfg(test)]
mod test {
    use super::FormItems;

    macro_rules! check_form {
        ($string:expr, $expected: expr) => ({
            let results: Vec<(&str, &str)> = FormItems($string).collect();
            assert_eq!($expected.len(), results.len());

            for i in 0..results.len() {
                let (expected_key, actual_key) = ($expected[i].0, results[i].0);
                let (expected_val, actual_val) = ($expected[i].1, results[i].1);

                assert!(expected_key == actual_key,
                    "key [{}] mismatch: expected {}, got {}",
                        i, expected_key, actual_key);

                assert!(expected_val == actual_val,
                    "val [{}] mismatch: expected {}, got {}",
                        i, expected_val, actual_val);
            }
        })
    }

    #[test]
    fn test_form_string() {
        check_form!("username=user&password=pass",
                    &[("username", "user"), ("password", "pass")]);

        check_form!("user=user&user=pass",
                    &[("user", "user"), ("user", "pass")]);

        check_form!("user=&password=pass",
                    &[("user", ""), ("password", "pass")]);

        check_form!("=&=", &[("", ""), ("", "")]);

        check_form!("a=b", &[("a", "b")]);

        check_form!("a=b&a", &[("a", "b")]);

        check_form!("a=b&a=", &[("a", "b"), ("a", "")]);
    }
}
