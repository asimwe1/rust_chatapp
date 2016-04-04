#![feature(plugin)]
#![plugin(rocket_macros)]

extern crate rocket;

mod files;

use rocket::Rocket;
use rocket::response::Redirect;
use rocket::Error;

#[route(GET, path = "/user/<username>")]
fn user_page(username: &str) -> String {
    format!("This is {}'s page.", username)
}

// #[derive(FromForm)] // FIXME: Make that happen.
struct UserLogin<'a> {
    username: &'a str,
    password: &'a str
}

fn form_items<'f>(string: &'f str, items: &mut [(&'f str, &'f str)]) -> usize {
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

trait FromForm<'f>: Sized {
    fn from_form_string(s: &'f str) -> Result<Self, Error>;
}

// FIXME: Add a 'FromFormValue' trait and use it on each field in the form
// structure. Should be pretty simple. Implement for all FromStr and then
// implement for OptionT: FromFormValue> so forms still exist. Maybe have a way
// for FromFormValue types to have an error that says why it didn't work. This
// will help for validation. IE, can have a type Range(1, 10) that returns an
// enum with one of: TooLow(isize), TooHigh(isize), etc.
impl<'f> FromForm<'f> for UserLogin<'f> {
    fn from_form_string(s: &'f str) -> Result<Self, Error> {
        let mut items = [("", ""); 2];
        let form_count = form_items(s, &mut items);
        if form_count != 2 {
            return Err(Error::BadParse);
        }

        let mut username = None;
        let mut password = None;
        for &(key, value) in &items {
            match key {
                "username" => username = Some(value),
                "password" => password = Some(value),
                _ => return Err(Error::BadParse)
            }
        }

        if username.is_none() || password.is_none() {
            return Err(Error::BadParse);
        }

        Ok(UserLogin {
            username: username.unwrap(),
            password: password.unwrap()
        })
    }
}

// TODO: Actually look at form parameters.
// FIXME: fn login<'a>(user: UserLogin<'a>)
#[route(POST, path = "/login", form = "<user>")]
fn login(user: UserLogin) -> Result<Redirect, String> {
    match user.username {
        "Sergio" => match user.password {
            "password" => Ok(Redirect::other("/user/Sergio")),
            _ => Err("Wrong password!".to_string())
        },
        _ => Err(format!("Unrecognized user, '{}'.", user.username))
    }
}

fn main() {
    let mut rocket = Rocket::new("localhost", 8000);
    rocket.mount("/", routes![files::index, files::files, user_page, login]);
    rocket.launch();
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
