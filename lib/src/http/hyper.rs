//! Re-exported hyper HTTP library types.
//!
//! ## Hyper
//!
//! All types that are re-exported from Hyper resides inside of this module.
//! These types will, with certainty, be removed with time, but they reside here
//! while necessary.

// TODO: Remove from Rocket in favor of a more flexible HTTP library.
pub use hyper::server::Request as HyperRequest;
pub use hyper::server::Response as HyperResponse;
pub use hyper::server::Server as HyperServer;
pub use hyper::server::Handler as HyperHandler;

pub use hyper::header::Headers as HyperHeaders;
pub use hyper::header::CookiePair as HyperCookiePair;
pub use hyper::header::CookieJar as HyperCookieJar;
pub use hyper::header::Cookie as HyperCookie;
pub use hyper::header::SetCookie as HyperSetCookie;

pub use hyper::method::Method as HyperMethod;
pub use hyper::uri::RequestUri as HyperRequestUri;
pub use hyper::net::Fresh as HyperFresh;
pub use hyper::net::HttpStream as HyperHttpStream;
pub use hyper::net::NetworkStream as HyperNetworkStream;
pub use hyper::http::h1::HttpReader as HyperHttpReader;
pub use hyper::header;

// This is okay for now.
pub use hyper::status::StatusCode;

// TODO: Remove from Rocket in favor of a more flexible HTTP library.
pub type FreshHyperResponse<'a> = self::HyperResponse<'a, self::HyperFresh>;

// TODO: Remove from Rocket in favor of a more flexible HTTP library.
use hyper::buffer::BufReader;
pub type HyperBodyReader<'a, 'b> =
    HyperHttpReader<&'a mut BufReader<&'b mut HyperNetworkStream>>;

