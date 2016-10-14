use outcome::Outcome;
use http::StatusCode;
use request::{Request, Data};

pub type DataOutcome<S, E> = Outcome<S, (StatusCode, E), Data>;

impl<S, E> DataOutcome<S, E> {
    #[inline(always)]
    pub fn of(result: Result<S, E>) -> Self {
        match result {
            Ok(val) => DataOutcome::success(val),
            Err(err) => DataOutcome::failure(StatusCode::InternalServerError, err)
        }
    }

    #[inline(always)]
    pub fn success(s: S) -> Self {
        Outcome::Success(s)
    }

    #[inline(always)]
    pub fn failure(status: StatusCode, e: E) -> Self {
        Outcome::Failure((status, e))
    }

    #[inline(always)]
    pub fn forward(data: Data) -> Self {
        Outcome::Forward(data)
    }
}

/// Trait used to derive an object from incoming request data.
pub trait FromData: Sized {
    type Error;

    fn from_data(request: &Request, data: Data) -> DataOutcome<Self, Self::Error>;
}

impl<T: FromData> FromData for Result<T, T::Error> {
    type Error = ();

    fn from_data(request: &Request, data: Data) -> DataOutcome<Self, Self::Error> {
        match T::from_data(request, data) {
            Outcome::Success(val) => DataOutcome::success(Ok(val)),
            Outcome::Failure((_, val)) => DataOutcome::success(Err(val)),
            Outcome::Forward(data) => DataOutcome::forward(data),
        }
    }
}

impl<T: FromData> FromData for Option<T> {
    type Error = ();

    fn from_data(request: &Request, data: Data) -> DataOutcome<Self, Self::Error> {
        match T::from_data(request, data) {
            Outcome::Success(val) => DataOutcome::success(Some(val)),
            Outcome::Failure(_) => DataOutcome::success(None),
            Outcome::Forward(_) => DataOutcome::success(None)
        }
    }
}
