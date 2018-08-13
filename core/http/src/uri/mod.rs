//! Types for URIs and traits for rendering URI components.

mod uri;
mod uri_display;
mod from_uri_param;
mod origin;
mod authority;
mod absolute;
mod segments;

pub use parse::uri::Error;

pub use self::uri::*;
pub use self::authority::*;
pub use self::origin::*;
pub use self::absolute::*;
pub use self::uri_display::*;
pub use self::from_uri_param::*;
pub use self::segments::*;
