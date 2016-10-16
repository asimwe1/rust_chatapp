#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;

use std::fmt;
use rocket::request::{Request, FromRequest, RequestOutcome};

#[derive(Debug)]
struct HeaderCount(usize);

impl fmt::Display for HeaderCount {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<'r> FromRequest<'r> for HeaderCount {
    type Error = ();
    fn from_request(request: &'r Request) -> RequestOutcome<Self, Self::Error> {
        RequestOutcome::success(HeaderCount(request.headers().len()))
    }
}

#[get("/")]
fn header_count(header_count: HeaderCount) -> String {
    format!("Your request contained {} headers!", header_count)
}

fn main() {
    rocket::ignite().mount("/", routes![header_count]).launch()
}

#[cfg(test)]
mod test {
    use super::rocket;
    use rocket::testing::MockRequest;
    use rocket::http::Method::*;

    fn test_header_count<'h>(headers: &[(&'h str, &'h str)]) {
        let rocket = rocket::ignite().mount("/", routes![super::header_count]);
        let req = MockRequest::new(Get, "/").headers(headers);
        let result = req.dispatch_with(&rocket);
        assert_eq!(result.unwrap(),
            format!("Your request contained {} headers!", headers.len()));
    }

    #[test]
    fn test_n_headers() {
        for i in 0..50 {
            let mut headers = vec![];
            for j in 0..i {
                let string = format!("{}", j);
                headers.push((string.clone(), string));
            }

            let h_strs: Vec<_> = headers.iter()
                .map(|&(ref a, ref b)| (a.as_str(), b.as_str()))
                .collect();

            test_header_count(&h_strs);
        }
    }
}
