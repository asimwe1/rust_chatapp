//! Types and traits for request parsing and handling.

mod request;
mod param;
mod form;
mod from_request;
mod state;

#[cfg(test)]
mod tests;

pub use self::request::Request;
pub use self::from_request::{FromRequest, Outcome};
pub use self::param::{FromParam, FromSegments};
pub use self::form::{Form, LenientForm, FormItems};
pub use self::form::{FromForm, FormError, FromFormValue, FormParseError, FormDataError};
pub use self::state::State;

#[doc(inline)]
pub use response::flash::FlashMessage;
