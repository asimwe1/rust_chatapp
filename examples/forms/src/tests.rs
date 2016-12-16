use super::rocket;
use rocket::testing::MockRequest;
use rocket::http::Method::*;
use rocket::http::ContentType;

fn test_login<F: Fn(String) -> bool>(username: &str, password: &str, age: isize, test: F) {
    let rocket = rocket::ignite().mount("/", routes![super::user_page, super::login]);
    let result = MockRequest::new(Post, "/login")
        .header(ContentType::Form)
        .body(&format!("username={}&password={}&age={}", username, password, age))
        .dispatch_with(&rocket)
        .unwrap_or("".to_string());
    assert!(test(result));
}

#[test]
fn test_good_login() {
    // TODO: Be able to check if it's a redirect, and process the redirect.
    test_login("Sergio", "password", 30, |s| s.is_empty());
}

#[test]
fn test_bad_login() {
    test_login("Sergio", "password", 20, |s| s == "Sorry, 20 is too young!");
    test_login("Sergio", "password", 200, |s| s == "Are you sure you're 200?");
    test_login("Sergio", "password", -100, |s| s == "'-100' is not a valid integer.");
    test_login("Sergio", "ok", 30, |s| s == "Wrong password!");
    test_login("Mike", "password", 30, |s| s == "Unrecognized user, 'Mike'.");
}

#[test]
fn test_bad_form() {
    // FIXME: Need to be able to examine the status.
    // test_login("Sergio&other=blah&", "password", 30, |s| s.contains("400 Bad Request"));
    test_login("Sergio&other=blah&", "password", 30, |s| s.is_empty());
}
