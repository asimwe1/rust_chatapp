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
/// for (key, value) in FormItems(form_string) {
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
/// for (key, value) in FormItems(form_string) {
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
