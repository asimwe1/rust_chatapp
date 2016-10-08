use std::io::Write;

use response::{ResponseOutcome, Outcome, Responder};
use http::hyper::{header, FreshHyperResponse};
use http::hyper::StatusCode;

pub struct Empty(StatusCode);

impl Empty {
    #[inline(always)]
    pub fn new(status: StatusCode) -> Empty {
        Empty(status)
    }

    #[inline(always)]
    pub fn not_found() -> Empty {
        Empty::new(StatusCode::NotFound)
    }

    #[inline(always)]
    pub fn server_error(reason: &str) -> Empty {
        warn_!("internal server error: {}", reason);
        Empty::new(StatusCode::InternalServerError)
    }

    #[inline(always)]
    pub fn bad_request(reason: &str) -> Empty {
        warn_!("internal server error: {}", reason);
        Empty::new(StatusCode::BadRequest)
    }
}

impl Responder for Empty {
    fn respond<'a>(&mut self, mut res: FreshHyperResponse<'a>) -> ResponseOutcome<'a> {
        res.headers_mut().set(header::ContentLength(0));
        *(res.status_mut()) = self.0;

        let mut stream = res.start().unwrap();
        stream.write_all(b"").unwrap();
        Outcome::Success
    }
}
