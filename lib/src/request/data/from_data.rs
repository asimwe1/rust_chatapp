use std::fmt::Debug;

use request::{Request, Data};

/// Trait used to derive an object from incoming request data.
pub trait FromData: Sized {
    type Error = ();
    fn from_data(request: &Request, data: Data) -> DataOutcome<Self, Self::Error>;
}

impl<T: FromData> FromData for Result<T, T::Error> {
    fn from_data(request: &Request, data: Data) -> DataOutcome<Self, Self::Error> {
        match T::from_data(request, data) {
            DataOutcome::Success(val) => DataOutcome::Success(Ok(val)),
            DataOutcome::Failure(val) => DataOutcome::Success(Err(val)),
            DataOutcome::Forward(data) => DataOutcome::Forward(data)
        }
    }
}

impl<T: FromData> FromData for Option<T> {
    fn from_data(request: &Request, data: Data) -> DataOutcome<Self, Self::Error> {
        match T::from_data(request, data) {
            DataOutcome::Success(val) => DataOutcome::Success(Some(val)),
            DataOutcome::Failure(_) => DataOutcome::Success(None),
            DataOutcome::Forward(data) => DataOutcome::Forward(data)
        }
    }
}


#[must_use]
pub enum DataOutcome<T, E> {
    /// Signifies that all processing completed successfully.
    Success(T),
    /// Signifies that some processing occurred that ultimately resulted in
    /// failure. As a result, no further processing can occur.
    Failure(E),
    /// Signifies that no processing occured and as such, processing can be
    /// forwarded to the next available target.
    Forward(Data),
}

impl<T, E: Debug> From<Result<T, E>> for DataOutcome<T, E> {
    fn from(result: Result<T, E>) -> Self {
        match result {
            Ok(val) => DataOutcome::Success(val),
            Err(e) => {
                error_!("{:?}", e);
                DataOutcome::Failure(e)
            }
        }
    }
}

impl<T> From<Option<T>> for DataOutcome<T, ()> {
    fn from(result: Option<T>) -> Self {
        match result {
            Some(val) => DataOutcome::Success(val),
            None => DataOutcome::Failure(())
        }
    }
}
