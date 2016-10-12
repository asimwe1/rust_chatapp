use response::{ResponseOutcome, Outcome, Responder};
use http::hyper::{FreshHyperResponse, StatusCode};

#[derive(Debug)]
pub struct Failure(pub StatusCode);

impl Responder for Failure {
    fn respond<'a>(&mut self, res: FreshHyperResponse<'a>) -> ResponseOutcome<'a> {
        Outcome::Forward((self.0, res))
    }
}
