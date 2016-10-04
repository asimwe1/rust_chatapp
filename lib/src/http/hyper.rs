// TODO: Removed from Rocket in favor of a more flexible HTTP library.
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
pub use hyper::header;

// This is okay.
pub use hyper::status::StatusCode;

// TODO: Removed from Rocket in favor of a more flexible HTTP library.
pub type FreshHyperResponse<'a> = self::HyperResponse<'a, self::HyperFresh>;
