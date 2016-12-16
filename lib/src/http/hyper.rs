//! Re-exported hyper HTTP library types.
//!
//! ## Hyper
//!
//! All types that are re-exported from Hyper resides inside of this module.
//! These types will, with certainty, be removed with time, but they reside here
//! while necessary.

// TODO: Remove from Rocket in favor of a more flexible HTTP library.
pub use hyper::server::Request as Request;
pub use hyper::server::Response as Response;
pub use hyper::server::Server as Server;
pub use hyper::server::Handler as Handler;

pub use hyper::header;
pub use hyper::mime;
pub use hyper::net;

pub use hyper::method::Method;
pub use hyper::status::StatusCode;
pub use hyper::uri::RequestUri;
pub use hyper::http::h1;
pub use hyper::buffer;

// TODO: Remove from Rocket in favor of a more flexible HTTP library.
pub type FreshResponse<'a> = self::Response<'a, self::net::Fresh>;
