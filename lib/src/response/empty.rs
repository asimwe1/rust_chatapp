use response::*;
use std::io::Write;

pub struct Empty(StatusCode);

impl Empty {
    pub fn new(status: StatusCode) -> Empty {
        Empty(status)
    }
}

impl Responder for Empty {
    fn respond<'a>(&mut self, mut res: FreshHyperResponse<'a>) -> Outcome<'a> {
        res.headers_mut().set(header::ContentLength(0));
        *(res.status_mut()) = self.0;

        let mut stream = res.start().unwrap();
        stream.write_all(b"").unwrap();
        Outcome::Complete
    }
}

pub struct Forward;

impl Responder for Forward {
    fn respond<'a>(&mut self, res: FreshHyperResponse<'a>) -> Outcome<'a> {
        Outcome::FailForward(res)
    }
}
