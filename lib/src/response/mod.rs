//! Types and traits to build and send responses.
//!
//! The return type of a Rocket handler can be any type that implements the
//! [Responder](trait.Responder.html) trait. This module contains several useful
//! types that implement this trait.

mod responder;
mod redirect;
mod with_status;
mod flash;
mod named_file;
mod stream;
mod response;
mod failure;

pub mod content;

pub use self::response::Response;
pub use self::responder::{Outcome, Responder};
pub use self::redirect::Redirect;
pub use self::with_status::StatusResponse;
pub use self::flash::Flash;
pub use self::named_file::NamedFile;
pub use self::stream::Stream;
pub use self::content::Content;
pub use self::failure::Failure;
