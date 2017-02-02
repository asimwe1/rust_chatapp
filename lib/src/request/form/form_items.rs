use memchr::memchr2;

/// Iterator over the key/value pairs of a given HTTP form string. You'll likely
/// want to use this if you're implementing [FromForm](trait.FromForm.html)
/// manually, for whatever reason, by iterating over the items in `form_string`.
///
/// **Note:** The returned key/value pairs are _not_ URL decoded. To URL decode
/// the raw strings, use `String::from_form_value`:
///
/// ```rust
/// use rocket::request::{FormItems, FromFormValue};
///
/// let form_string = "greeting=Hello%2C+Mark%21&username=jake%2Fother";
/// for (key, value) in FormItems::from(form_string) {
///     let decoded_value = String::from_form_value(value);
///     match key {
///         "greeting" => assert_eq!(decoded_value, Ok("Hello, Mark!".into())),
///         "username" => assert_eq!(decoded_value, Ok("jake/other".into())),
///         _ => unreachable!()
///     }
/// }
/// ```
///
/// # Examples
///
/// `FormItems` can be used directly as an iterator:
///
/// ```rust
/// use rocket::request::FormItems;
///
/// // prints "greeting = hello" then "username = jake"
/// let form_string = "greeting=hello&username=jake";
/// for (key, value) in FormItems::from(form_string) {
///     println!("{} = {}", key, value);
/// }
/// ```
///
/// This is the same example as above, but the iterator is used explicitly.
///
/// ```rust
/// use rocket::request::FormItems;
///
/// let form_string = "greeting=hello&username=jake";
/// let mut items = FormItems::from(form_string);
/// assert_eq!(items.next(), Some(("greeting", "hello")));
/// assert_eq!(items.next(), Some(("username", "jake")));
/// assert_eq!(items.next(), None);
/// assert!(items.completed());
/// ```
pub struct FormItems<'f> {
    string: &'f str,
    next_index: usize
}

impl<'f> FormItems<'f> {
    #[inline]
    pub fn completed(&self) -> bool {
        self.next_index >= self.string.len()
    }

    pub fn exhausted(&mut self) -> bool {
        while let Some(_) = self.next() {  }
        self.completed()
    }

    #[inline]
    #[doc(hidden)]
    pub fn mark_complete(&mut self) {
        self.next_index = self.string.len()
    }

    #[inline]
    pub fn inner_str(&self) -> &'f str {
        self.string
    }
}

impl<'f> From<&'f str> for FormItems<'f> {
    fn from(string: &'f str) -> FormItems<'f> {
        FormItems {
            string: string,
            next_index: 0
        }
    }
}

impl<'f> Iterator for FormItems<'f> {
    type Item = (&'f str, &'f str);

    fn next(&mut self) -> Option<Self::Item> {
        let s = &self.string[self.next_index..];
        let (key, rest) = match memchr2(b'=', b'&', s.as_bytes()) {
            Some(i) if s.as_bytes()[i] == b'=' => (&s[..i], &s[(i + 1)..]),
            Some(_) => return None,
            None => return None,
        };

        if key.is_empty() {
            return None;
        }

        let (value, consumed) = match rest.find('&') {
            Some(index) => (&rest[..index], index + 1),
            None => (rest, rest.len()),
        };

        self.next_index += key.len() + 1 + consumed;
        Some((key, value))
    }
}


#[cfg(test)]
mod test {
    use super::FormItems;

    macro_rules! check_form {
        (@opt $string:expr, $expected:expr) => ({
            let mut items = FormItems::from($string);
            let results: Vec<_> = items.by_ref().collect();
            if let Some(expected) = $expected {
                assert_eq!(expected.len(), results.len());

                for i in 0..results.len() {
                    let (expected_key, actual_key) = (expected[i].0, results[i].0);
                    let (expected_val, actual_val) = (expected[i].1, results[i].1);

                    assert!(expected_key == actual_key,
                            "key [{}] mismatch: expected {}, got {}",
                            i, expected_key, actual_key);

                    assert!(expected_val == actual_val,
                            "val [{}] mismatch: expected {}, got {}",
                            i, expected_val, actual_val);
                }
            } else {
                assert!(!items.exhausted());
            }
        });
        (@bad $string:expr) => (check_form!(@opt $string, None : Option<&[(&str, &str)]>));
        ($string:expr, $expected:expr) => (check_form!(@opt $string, Some($expected)));
    }

    #[test]
    fn test_form_string() {
        check_form!("username=user&password=pass",
                    &[("username", "user"), ("password", "pass")]);

        check_form!("user=user&user=pass",
                    &[("user", "user"), ("user", "pass")]);

        check_form!("user=&password=pass",
                    &[("user", ""), ("password", "pass")]);

        check_form!("a=b", &[("a", "b")]);

        check_form!("user=", &[("user", "")]);
        check_form!("user=&", &[("user", "")]);
        check_form!("a=b&a=", &[("a", "b"), ("a", "")]);

        check_form!(@bad "user=&password");
        check_form!(@bad "a=b&a");
        check_form!(@bad "=");
        check_form!(@bad "&");
        check_form!(@bad "=&");
        check_form!(@bad "&=&");
        check_form!(@bad "=&=");
    }
}
