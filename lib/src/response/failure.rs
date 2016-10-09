use response::{ResponseOutcome, Outcome, Responder};
use http::hyper::{FreshHyperResponse, StatusCode};

pub struct Failure(StatusCode);

impl Failure {
    #[inline(always)]
    pub fn new(status: StatusCode) -> Failure {
        Failure(status)
    }
}

impl Responder for Failure {
    fn respond<'a>(&mut self, res: FreshHyperResponse<'a>) -> ResponseOutcome<'a> {
        Outcome::Forward((self.0, res))
    }
}
