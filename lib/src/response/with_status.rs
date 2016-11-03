use response::{Responder, Outcome};
use http::hyper::{StatusCode, FreshHyperResponse};

/// Responds to the client using a wrapped `Responder` and a given
/// `StatusCode`.
#[derive(Debug)]
pub struct StatusResponse<R: Responder> {
    status: StatusCode,
    responder: R,
}

impl<R: Responder> StatusResponse<R> {
    /// Creates a response with the given status code and underyling responder.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket::response::StatusResponse;
    /// use rocket::http::StatusCode;
    ///
    /// let response = StatusResponse::new(StatusCode::ImATeapot, "Hi!");
    /// ```
    pub fn new(status: StatusCode, responder: R) -> StatusResponse<R> {
        StatusResponse {
            status: status,
            responder: responder,
        }
    }
}

/// Sets the status code of the response and then delegates the remainder of the
/// response to the wrapped responder.
impl<R: Responder> Responder for StatusResponse<R> {
    fn respond<'b>(&mut self, mut res: FreshHyperResponse<'b>) -> Outcome<'b> {
        *(res.status_mut()) = self.status;
        self.responder.respond(res)
    }
}
