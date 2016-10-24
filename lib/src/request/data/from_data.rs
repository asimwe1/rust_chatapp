use outcome::Outcome;
use http::StatusCode;
use request::{Request, Data};

/// Type alias for the `Outcome` of a `FromData` conversion.
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
///
/// Types that implement this trait can be used as a target for the `data =
/// "<param>"` route parmater, as illustrated below:
///
/// ```rust,ignore
/// #[post("/submit", data = "<var>")]
/// fn submit(var: T) -> ... { ... }
/// ```
///
/// In this example, `T` can be any type that implements `FromData.`
///
/// # Outcomes
///
/// The returned [Outcome](/rocket/outcome/index.html) of a `from_data` call
/// determines how the incoming request will be processed.
///
/// * **Success**(S)
///
///   If the `Outcome` is `Success`, then the `Success` value will be used as
///   the value for the data parameter.  As long as all other parsed types
///   succeed, the request will be handled by the requesting handler.
///
/// * **Failure**(StatusCode, E)
///
///   If the `Outcome` is `Failure`, the request will fail with the given status
///   code and error. The designated error
///   [Catcher](/rocket/struct.Catcher.html) will be used to respond to the
///   request. Note that users can request types of `Result<S, E>` and
///   `Option<S>` to catch `Failure`s and retrieve the error value.
///
/// * **Forward**(Data)
///
///   If the `Outcome` is `Forward`, the request will be forwarded to the next
///   matching request. This requires that no data has been read from the `Data`
///   parameter. Note that users can request an `Option<S>` to catch `Forward`s.
pub trait FromData: Sized {
    /// The associated error to be returned when parsing fails.
    type Error;

    /// Parses an instance of `Self` from the incoming request body data.
    ///
    /// If the parse is successful, an outcome of `Success` is returned. If the
    /// data does not correspond to the type of `Self`, `Forward` is returned.
    /// If parsing fails, `Failure` is returned.
    fn from_data(request: &Request, data: Data) -> DataOutcome<Self, Self::Error>;
}

/// The identity implementation of `FromData`. Always returns `Success`.
impl FromData for Data {
    type Error = ();
    fn from_data(_: &Request, data: Data) -> DataOutcome<Self, Self::Error> {
        DataOutcome::success(data)
    }
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
