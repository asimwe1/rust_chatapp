//! Types and traits to build and send responses.
//!
//! The return type of a Rocket handler can be any type that implements the
//! [Responder](trait.Responder.html) trait. This module contains several such
//! types.
//!
//! # Composing
//!
//! Many of the built-in `Responder` types _chain_ responses: they take in
//! another `Responder` and simply add, remove, or change information in the
//! response. In other words, many `Responder` types are built to compose well.
//! As a result, you'll often have types of the form `A<B<C>>` consisting of
//! three `Responder`s `A`, `B`, and `C`. This is normal and encouraged as the
//! type names typically illustrate the intended response.

mod responder;
mod redirect;
mod flash;
mod named_file;
mod stream;
mod response;
mod failure;

pub mod content;
pub mod status;

pub use self::response::{Response, Body, DEFAULT_CHUNK_SIZE};
pub use self::responder::Responder;
pub use self::redirect::Redirect;
pub use self::flash::Flash;
pub use self::named_file::NamedFile;
pub use self::stream::Stream;
pub use self::content::Content;
pub use self::failure::Failure;

pub type Result<'r> = ::std::result::Result<self::Response<'r>, ::http::Status>;
