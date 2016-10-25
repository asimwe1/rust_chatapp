//! Types and traits for request parsing and handling.
//!
//! # Request and Data
//!
//! The [Request](struct.Request.html) and [Data](struct.Data.html) types
//! contain all of the available information for an incoming request. The
//! `Request` types contains all information _except_ the body, which is
//! contained in the `Data` type.
//!
//! # Code Generation Conversion Traits
//!
//! This module contains the core code generation data conversion traits. These
//! traits are used by Rocket's code generation facilities to automatically
//! derive values from incoming data based on the signature of a request
//! handler.

mod request;
mod param;
mod form;
mod data;
mod from_request;

pub use self::request::Request;
pub use self::from_request::{FromRequest, Outcome};
pub use self::param::{FromParam, FromSegments};
pub use self::form::{Form, FromForm, FromFormValue, FormItems};
pub use self::data::{Data, FromData, DataOutcome};
