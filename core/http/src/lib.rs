#![warn(rust_2018_idioms)]
#![warn(missing_docs)]

//! Types that map to concepts in HTTP.
//!
//! This module exports types that map to HTTP concepts or to the underlying
//! HTTP library when needed.

#[macro_use]
extern crate pear;

pub mod uri;
pub mod ext;

#[macro_use]
mod header;
mod method;
mod status;
mod raw_str;
mod parse;

/// Case-preserving, ASCII case-insensitive string types.
///
/// An _uncased_ string is case-preserving. That is, the string itself contains
/// cased characters, but comparison (including ordering, equality, and hashing)
/// is ASCII case-insensitive. **Note:** the `alloc` feature _is_ enabled.
pub mod uncased {
    #[doc(inline)] pub use uncased::*;
}

// Types that we expose for use _only_ by core. Please don't use this.
#[doc(hidden)]
#[path = "."]
pub mod private {
    pub use crate::parse::Indexed;
}

pub use crate::method::Method;
pub use crate::status::{Status, StatusClass};
pub use crate::raw_str::{RawStr, RawStrBuf};
pub use crate::header::*;
