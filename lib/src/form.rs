use std::str::FromStr;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddrV4, SocketAddrV6, SocketAddr};
use url;

use error::Error;

pub trait FromForm<'f>: Sized {
    fn from_form_string(s: &'f str) -> Result<Self, Error>;
}

// This implementation should only be ued during debugging!
#[doc(hidden)]
impl<'f> FromForm<'f> for &'f str {
    fn from_form_string(s: &'f str) -> Result<Self, Error> {
        Ok(s)
    }
}

pub trait FromFormValue<'v>: Sized {
    type Error;

    fn parse(v: &'v str) -> Result<Self, Self::Error>;

    /// Returns a default value to be used when the form field does not exist.
    /// If this returns None, then the field is required. Otherwise, this should
    /// return Some(default_value).
    fn default() -> Option<Self> {
        None
    }
}

impl<'v> FromFormValue<'v> for &'v str {
    type Error = Error;

    fn parse(v: &'v str) -> Result<Self, Self::Error> {
        Ok(v)
    }
}

impl<'v> FromFormValue<'v> for String {
    type Error = &'v str;

    // This actually parses the value according to the standard.
    fn parse(v: &'v str) -> Result<Self, Self::Error> {
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

    fn parse(v: &'v str) -> Result<Self, Self::Error> {
        match v {
            "on" | "true" => Ok(true),
            "off" | "false" => Ok(false),
            _ => Err(v)
        }
    }
}

macro_rules! impl_with_fromstr {
    ($($T:ident),+) => ($(
        impl<'v> FromFormValue<'v> for $T {
            type Error = &'v str;
            fn parse(v: &'v str) -> Result<Self, Self::Error> {
                $T::from_str(v).map_err(|_| v)
            }
        }
    )+)
}

impl_with_fromstr!(f32, f64, isize, i8, i16, i32, i64, usize, u8, u16, u32, u64,
    IpAddr, Ipv4Addr, Ipv6Addr, SocketAddrV4, SocketAddrV6, SocketAddr);

impl<'v, T: FromFormValue<'v>> FromFormValue<'v> for Option<T> {
    type Error = Error;

    fn parse(v: &'v str) -> Result<Self, Self::Error> {
        match T::parse(v) {
            Ok(v) => Ok(Some(v)),
            Err(_) => Ok(None)
        }
    }

    fn default() -> Option<Option<T>> {
        Some(None)
    }
}

// TODO: Add more useful implementations (range, regex, etc.).
impl<'v, T: FromFormValue<'v>> FromFormValue<'v> for Result<T, T::Error> {
    type Error = Error;

    fn parse(v: &'v str) -> Result<Self, Self::Error> {
        match T::parse(v) {
            ok@Ok(_) => Ok(ok),
            e@Err(_) => Ok(e)
        }
    }
}

pub struct FormItems<'f>(pub &'f str);

impl<'f> Iterator for FormItems<'f> {
    type Item = (&'f str, &'f str);

    fn next(&mut self) -> Option<Self::Item> {
        let string = self.0;
        let (key, rest) = match string.find('=') {
            Some(index) => (&string[..index], &string[(index + 1)..]),
            None => return None
        };

        let (value, remainder) = match rest.find('&') {
            Some(index) => (&rest[..index], &rest[(index + 1)..]),
            None => (rest, "")
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
        let results = &[("username", "user"), ("password", "pass")];
        check_form!("username=user&password=pass", results);

        let results = &[("user", "user"), ("user", "pass")];
        check_form!("user=user&user=pass", results);

        let results = &[("user", ""), ("password", "pass")];
        check_form!("user=&password=pass", results);

        let results = &[("", ""), ("", "")];
        check_form!("=&=", results);

        let results = &[("a", "b")];
        check_form!("a=b", results);

        let results = &[("a", "b")];
        check_form!("a=b&a", results);

        let results = &[("a", "b"), ("a", "")];
        check_form!("a=b&a=", results);
    }
}
