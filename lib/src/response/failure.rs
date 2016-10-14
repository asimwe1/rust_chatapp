use response::{ResponseOutcome, Responder};
use http::hyper::{FreshHyperResponse, StatusCode};

#[derive(Debug)]
pub struct Failure(pub StatusCode);

impl Responder for Failure {
    fn respond<'a>(&mut self, res: FreshHyperResponse<'a>) -> ResponseOutcome<'a> {
        ResponseOutcome::forward(self.0, res)
    }
}
