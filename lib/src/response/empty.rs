use std::io::Write;

use response::{ResponseOutcome, Outcome, Responder};
use http::hyper::{header, FreshHyperResponse};
use http::hyper::StatusCode;

pub struct Empty(StatusCode);

impl Empty {
    pub fn new(status: StatusCode) -> Empty {
        Empty(status)
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

pub struct Forward;

impl Responder for Forward {
    fn respond<'a>(&mut self, res: FreshHyperResponse<'a>) -> ResponseOutcome<'a> {
        Outcome::FailForward(res)
    }
}
