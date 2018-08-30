#![cfg_attr(test, feature(plugin, decl_macro))]
#![cfg_attr(test, plugin(rocket_codegen))]
#![feature(proc_macro_non_items)]

#[macro_use] extern crate rocket;
extern crate rocket_contrib;

#[cfg(feature = "space_helmet")]
extern crate time;

#[cfg(feature = "space_helmet")]
mod space_helmet_tests {
    use rocket;
    use rocket::http::uri::Uri;
    use rocket::http::Status;
    use rocket::local::Client;
    use rocket_contrib::space_helmet::*;
    use time::Duration;

    #[get("/")]
    fn hello() -> &'static str {
        "Hello, world!"
    }

    macro_rules! check_header {
        ($response:ident, $header_name:expr, $header_param:expr) => {
            match $response.headers().get_one($header_name) {
                Some(string) => assert_eq!(string, $header_param),
                None => panic!("missing header parameters")
            }
        };
    }

    #[test]
    fn default_headers_test() {
        let helmet = SpaceHelmet::new();
        let rocket = rocket::ignite().mount("/", routes![hello]).attach(helmet);
        let client = Client::new(rocket).unwrap();
        let mut response = client.get("/").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.body_string(), Some("Hello, world!".into()));
        check_header!(response, "X-XSS-Protection", "1; mode=block");
        check_header!(response, "X-Frame-Options", "SAMEORIGIN");
        check_header!(response, "X-Content-Type-Options", "nosniff");
    }

    #[test]
    fn additional_headers_test() {
        let helmet = SpaceHelmet::new()
            .hsts(HstsPolicy::default())
            .expect_ct(ExpectCtPolicy::default())
            .referrer_policy(ReferrerPolicy::default());
        let rocket = rocket::ignite().mount("/", routes![hello]).attach(helmet);
        let client = Client::new(rocket).unwrap();
        let mut response = client.get("/").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.body_string(), Some("Hello, world!".into()));
        check_header!(
            response,
            "Strict-Transport-Security",
            format!("max-age={}", Duration::weeks(52).num_seconds())
        );
        check_header!(
            response,
            "Expect-CT",
            format!("max-age={}, enforce", Duration::days(30).num_seconds())
        );
        check_header!(response, "Referrer-Policy", "no-referrer");
    }

    #[test]
    fn uri_test() {
        let allow_uri = Uri::parse("https://www.google.com").unwrap();
        let report_uri = Uri::parse("https://www.google.com").unwrap();
        let enforce_uri = Uri::parse("https://www.google.com").unwrap();
        let helmet = SpaceHelmet::new()
            .frameguard(FramePolicy::AllowFrom(allow_uri))
            .xss_protect(XssPolicy::EnableReport(report_uri))
            .expect_ct(ExpectCtPolicy::ReportAndEnforce(Duration::seconds(30), enforce_uri));
        let rocket = rocket::ignite().mount("/", routes![hello]).attach(helmet);
        let client = Client::new(rocket).unwrap();
        let response = client.get("/").dispatch();
        check_header!(
            response,
            "X-Frame-Options",
            "ALLOW-FROM https://www.google.com"
        );
        check_header!(
            response,
            "X-XSS-Protection",
            "1; report=https://www.google.com"
        );
        check_header!(
            response,
            "Expect-CT",
            "max-age=30, enforce, report-uri=\"https://www.google.com\""
        );
    }
}
