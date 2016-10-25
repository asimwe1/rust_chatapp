use outcome::Outcome;
use response::{self, Responder};
use http::hyper::{FreshHyperResponse, StatusCode};

#[derive(Debug)]
pub struct Failure(pub StatusCode);

impl Responder for Failure {
    fn respond<'a>(&mut self, res: FreshHyperResponse<'a>) -> response::Outcome<'a> {
        Outcome::Forward((self.0, res))
    }
}
