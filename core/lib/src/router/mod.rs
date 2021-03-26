//! Routing types: [`Route`] and [`RouteUri`].

mod route;
mod segment;
mod uri;
mod router;
mod collider;

pub(crate) use router::*;

pub use route::Route;
pub use collider::Collide;
pub use uri::RouteUri;
