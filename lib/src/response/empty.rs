use response::*;
use std::io::Write;

pub struct Empty(StatusCode);

impl Empty {
    pub fn new(status: StatusCode) -> Empty {
        Empty(status)
    }
}

impl Responder for Empty {
    fn respond<'a>(&mut self, mut res: HypResponse<'a, HypFresh>) {
        res.headers_mut().set(header::ContentLength(0));
        *(res.status_mut()) = self.0;

        let mut stream = res.start().unwrap();
        stream.write_all(b"").unwrap();
    }
}
