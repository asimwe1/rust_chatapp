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
pub use self::form::{Form, FormError, LenientForm, FromForm, FromFormValue, FormItems};
pub use self::state::State;

#[doc(inline)]
pub use response::flash::FlashMessage;
