use std::str::FromStr;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddrV4, SocketAddrV6, SocketAddr};

use error::Error;

pub trait FromForm<'f>: Sized {
    fn from_form_string(s: &'f str) -> Result<Self, Error>;
}

pub trait FromFormValue<'v>: Sized {
    type Error;

    fn parse(v: &'v str) -> Result<Self, Self::Error>;
}

impl<'v> FromFormValue<'v> for &'v str {
    type Error = Error;

    fn parse(v: &'v str) -> Result<Self, Self::Error> {
        Ok(v)
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
       bool, String, IpAddr, Ipv4Addr, Ipv6Addr, SocketAddrV4, SocketAddrV6,
       SocketAddr);

impl<'v, T: FromFormValue<'v>> FromFormValue<'v> for Option<T> {
    type Error = Error;

    fn parse(v: &'v str) -> Result<Self, Self::Error> {
        match T::parse(v) {
            Ok(v) => Ok(Some(v)),
            Err(_) => Ok(None)
        }
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

pub fn form_items<'f>(string: &'f str, items: &mut [(&'f str, &'f str)]) -> usize {
    let mut param_num = 0;
    let mut rest = string;
    while rest.len() > 0 && param_num < items.len() {
        let (key, remainder) = match rest.find('=') {
            Some(index) => (&rest[..index], &rest[(index + 1)..]),
            None => return param_num
        };

        rest = remainder;
        let (value, remainder) = match rest.find('&') {
            Some(index) => (&rest[..index], &rest[(index + 1)..]),
            None => (rest, "")
        };

        rest = remainder;
        items[param_num] = (key, value);
        param_num += 1;
    }

    param_num
}

#[cfg(test)]
mod test {
    use super::form_items;

    macro_rules! check_form {
        ($string:expr, $expected: expr) => ({
            let mut output = Vec::with_capacity($expected.len());
            unsafe { output.set_len($expected.len()); }

            let results = output.as_mut_slice();
            assert_eq!($expected.len(), form_items($string, results));

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
