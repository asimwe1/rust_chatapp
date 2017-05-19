use super::rocket;
use rocket::testing::MockRequest;
use rocket::http::Method::*;
use rocket::http::Status;
use rocket::Response;
use rocket_contrib::Template;

const TEMPLATE_ROOT: &'static str = "templates/";

macro_rules! run_test {
    ($req:expr, $test_fn:expr) => ({
        let rocket = rocket();
        let mut req = $req;
        $test_fn(req.dispatch_with(&rocket));
    })
}

#[test]
fn test_root() {
    // Check that the redirect works.
    for method in &[Get, Head] {
        let req = MockRequest::new(*method, "/");
        run_test!(req, |mut response: Response| {
            assert_eq!(response.status(), Status::SeeOther);
            assert!(response.body().is_none());

            let location: Vec<_> = response.headers().get("Location").collect();
            assert_eq!(location, vec!["/hello/Unknown"]);
        });
    }

    // Check that other request methods are not accepted (and instead caught).
    for method in &[Post, Put, Delete, Options, Trace, Connect, Patch] {
        let req = MockRequest::new(*method, "/");
        run_test!(req, |mut response: Response| {
            let mut map = ::std::collections::HashMap::new();
            map.insert("path", "/");
            let expected = Template::show(TEMPLATE_ROOT, "error/404", &map).unwrap();

            assert_eq!(response.status(), Status::NotFound);
            assert_eq!(response.body_string(), Some(expected));
        });
    }
}

#[test]
fn test_name() {
    // Check that the /hello/<name> route works.
    let req = MockRequest::new(Get, "/hello/Jack");
    run_test!(req, |mut response: Response| {
        assert_eq!(response.status(), Status::Ok);

        let context = super::TemplateContext {
            name: "Jack".to_string(),
            items: vec!["One", "Two", "Three"].iter().map(|s| s.to_string()).collect()
        };

        let expected = Template::show(TEMPLATE_ROOT, "index", &context).unwrap();
        assert_eq!(response.body_string(), Some(expected));
    });
}

#[test]
fn test_404() {
    // Check that the error catcher works.
    let req = MockRequest::new(Get, "/hello/");
    run_test!(req, |mut response: Response| {
        assert_eq!(response.status(), Status::NotFound);

        let mut map = ::std::collections::HashMap::new();
        map.insert("path", "/hello/");

        let expected = Template::show(TEMPLATE_ROOT, "error/404", &map).unwrap();
        assert_eq!(response.body_string(), Some(expected));
    });
}
