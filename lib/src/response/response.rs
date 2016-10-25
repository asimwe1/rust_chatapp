use data::Data;
use outcome::{self, Outcome};
use http::hyper::StatusCode;
use response::{Responder, StatusResponse};

pub type Response<'a> = outcome::Outcome<Box<Responder + 'a>, StatusCode, Data>;

impl<'a> Response<'a> {
    #[inline(always)]
    pub fn success<T: Responder + 'a>(responder: T) -> Response<'a> {
        Outcome::Success(Box::new(responder))
    }

    #[inline(always)]
    pub fn failure(code: StatusCode) -> Response<'static> {
        Outcome::Failure(code)
    }

    #[inline(always)]
    pub fn forward(data: Data) -> Response<'static> {
        Outcome::Forward(data)
    }

    #[inline(always)]
    pub fn with_raw_status<T: Responder + 'a>(status: u16, body: T) -> Response<'a> {
        let status_code = StatusCode::from_u16(status);
        Response::success(StatusResponse::new(status_code, body))
    }

    #[doc(hidden)]
    #[inline(always)]
    pub fn responder(self) -> Option<Box<Responder + 'a>> {
        self.succeeded()
    }
}
