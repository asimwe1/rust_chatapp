use response::{Response, Responder};
use http::Status;

/// A failing response; simply forwards to the catcher for the given
/// `Status`.
#[derive(Debug)]
pub struct Failure(pub Status);

impl<'r> Responder<'r> for Failure {
    fn respond(self) -> Result<Response<'r>, Status> {
        Err(self.0)
    }
}
