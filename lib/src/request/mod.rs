//! Types and traits that deal with request parsing and handling.

mod request;
mod param;
mod form;
mod data;
mod from_request;

pub use self::request::Request;
pub use self::from_request::FromRequest;
pub use self::param::{FromParam, FromSegments};
pub use self::form::{Form, FromForm, FromFormValue, FormItems};
pub use self::data::{Data, FromData, DataOutcome};
