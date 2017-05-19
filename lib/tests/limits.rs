#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]

extern crate rocket;

use rocket::request::Form;

#[derive(FromForm)]
struct Simple {
    value: String
}

#[post("/", data = "<form>")]
fn index(form: Form<Simple>) -> String {
    form.into_inner().value
}

mod limits_tests {
    use rocket;
    use rocket::config::{Environment, Config, Limits};
    use rocket::testing::MockRequest;
    use rocket::http::Method::*;
    use rocket::http::{Status, ContentType};

    fn rocket_with_forms_limit(limit: u64) -> rocket::Rocket {
        let config = Config::build(Environment::Development)
            .limits(Limits::default().add("forms", limit))
            .unwrap();

        rocket::custom(config, true).mount("/", routes![super::index])
    }

    #[test]
    fn large_enough() {
        let rocket = rocket_with_forms_limit(128);
        let mut req = MockRequest::new(Post, "/")
            .body("value=Hello+world")
            .header(ContentType::Form);

        let mut response = req.dispatch_with(&rocket);
        assert_eq!(response.body_string(), Some("Hello world".into()));
    }

    #[test]
    fn just_large_enough() {
        let rocket = rocket_with_forms_limit(17);
        let mut req = MockRequest::new(Post, "/")
            .body("value=Hello+world")
            .header(ContentType::Form);

        let mut response = req.dispatch_with(&rocket);
        assert_eq!(response.body_string(), Some("Hello world".into()));
    }

    #[test]
    fn much_too_small() {
        let rocket = rocket_with_forms_limit(4);
        let mut req = MockRequest::new(Post, "/")
            .body("value=Hello+world")
            .header(ContentType::Form);

        let response = req.dispatch_with(&rocket);
        assert_eq!(response.status(), Status::BadRequest);
    }

    #[test]
    fn contracted() {
        let rocket = rocket_with_forms_limit(10);
        let mut req = MockRequest::new(Post, "/")
            .body("value=Hello+world")
            .header(ContentType::Form);

        let mut response = req.dispatch_with(&rocket);
        assert_eq!(response.body_string(), Some("Hell".into()));
    }
}
