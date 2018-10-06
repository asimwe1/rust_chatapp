#![feature(crate_visibility_modifier)]
#![feature(never_type)]
#![feature(doc_cfg)]

// TODO: Version URLs.
#![doc(html_root_url = "https://api.rocket.rs")]

//! This crate contains officially sanctioned contributor libraries that provide
//! functionality commonly used by Rocket applications.
//!
//! These libraries are always kept in-sync with the core Rocket library. They
//! provide common, but not fundamental, abstractions to be used by Rocket
//! applications.
//!
//! Each module in this library is held behind a feature flag, with the most
//! common modules exposed by default. The present feature list is below, with
//! an asterisk next to the features that are enabled by default:
//!
//! * [json*](Json)
//! * [static_files*](static_files)
//! * [msgpack](MsgPack)
//! * [handlebars_templates](Template)
//! * [tera_templates](Template)
//! * [uuid](Uuid)
//! * [${database}_pool](databases)
//!
//! The recommend way to include features from this crate via Cargo in your
//! project is by adding a `[dependencies.rocket_contrib]` section to your
//! `Cargo.toml` file, setting `default-features` to false, and specifying
//! features manually. For example, to use the JSON module, you would add:
//!
//! ```toml
//! [dependencies.rocket_contrib]
//! version = "*"
//! default-features = false
//! features = ["json"]
//! ```
//!
//! This crate is expected to grow with time, bringing in outside crates to be
//! officially supported by Rocket.

#[allow(unused_imports)] #[macro_use] extern crate log;
#[allow(unused_imports)] #[macro_use] extern crate rocket;

#[cfg(feature = "serde")]
extern crate serde;

#[cfg(feature = "json")]
extern crate serde_json;

#[cfg(feature = "json")]
pub use serde_json::{json_internal, json_internal_vec};

#[cfg(feature = "handlebars_templates")]
pub extern crate handlebars;

#[cfg(feature = "tera_templates")]
pub extern crate tera;

#[cfg(feature = "json")]
#[cfg_attr(feature = "json", macro_use)]
#[doc(hidden)]
pub mod json;

#[cfg(feature = "json")]
pub use json::{Json, JsonError, JsonValue};

#[cfg(feature = "msgpack")]
#[doc(hidden)]
pub mod msgpack;

#[cfg(feature = "msgpack")]
pub use msgpack::{MsgPack, MsgPackError};

#[cfg(feature = "templates")]
mod templates;

#[cfg(feature = "templates")]
pub use templates::{Engines, Template, TemplateMetadata};

#[cfg(feature = "uuid")]
mod uuid;

#[cfg(feature = "uuid")]
pub use uuid::{Uuid, UuidParseError};

#[cfg(feature = "static_files")]
pub mod static_files;

#[cfg(feature = "database_pool")]
pub mod databases;

#[cfg(feature = "database_pool_codegen")]
#[allow(unused_imports)]
#[macro_use]
extern crate rocket_contrib_codegen;

#[cfg(feature = "database_pool_codegen")]
#[doc(hidden)]
pub use rocket_contrib_codegen::*;
