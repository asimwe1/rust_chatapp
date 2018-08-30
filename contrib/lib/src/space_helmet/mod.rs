//! [`SpaceHelmet`] is a [`Fairing`](/rocket/fairing/trait.Fairing.html) that
//! turns on browsers security features by adding HTTP headers to all outgoing
//! responses.
//!
//! It provides a typed interface for http security headers and takes some
//! inspiration from [helmet](https://helmetjs.github.io/), a similar piece
//! of middleware for [express](https://expressjs.com).
//!
//! ### What it supports
//!
//! | HTTP Header                 | Description                            | Method                             | Enabled by Default? |
//! | --------------------------- | -------------------------------------- | ---------------------------------- | ------------------- |
//! | [X-XSS-Protection]          | Prevents some reflected XSS attacks.   | [`SpaceHelmet::xss_protect()`]     | ✔                   |
//! | [X-Content-Type-Options]    | Prevents client sniffing of MIME type. | [`SpaceHelmet::no_sniff()`]        | ✔                   |
//! | [X-Frame-Options]           | Prevents [clickjacking].                 | [`SpaceHelmet::frameguard()`]      | ✔                   |
//! | [Strict-Transport-Security] | Enforces strict use of HTTPS.          | [`SpaceHelmet::hsts()`]            | ?                   |
//! | [Expect-CT]                 | Enables certificate transparency.      | [`SpaceHelmet::expect_ct()`]       | ✗                   |
//! | [Referrer-Policy]           | Enables referrer policy.               | [`SpaceHelmet::referrer_policy()`] | ✗                   |
//!
//! [X-XSS-Protection]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/X-XSS-Protection
//! [X-Content-Type-Options]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/X-Content-Type-Options
//! [X-Frame-Options]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/X-Frame-Options
//! [Strict-Transport-Security]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Strict-Transport-Security
//! [Expect-CT]:  https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Expect-CT
//! [Referrer-Policy]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Referrer-Policy
//!  [clickjacking]: https://en.wikipedia.org/wiki/Clickjacking
//! _? If tls is enabled when a [Rocket](/rocket/struct.Rocket.html) is
//! [launched()'ed](/rocket/fairing/trait.Fairing.html)
//! in a non-development environment e.g.  [Staging](/rocket/struct.Config.html#method.staging)
//! or [Production](/rocket/struct.Config.html#method.production), `SpaceHelmet` enables hsts with its
//! default policy and outputs a warning._
//!
//! ### Examples
//!
//! To apply the headers that are enabled by default, just create a new [`SpaceHelmet`] instance
//! and attach before launch.
//!
//! ```rust
//! # extern crate rocket;
//! # extern crate rocket_contrib;
//! use rocket_contrib::space_helmet::{SpaceHelmet};
//!
//! let rocket = rocket::ignite().attach(SpaceHelmet::new());
//! ```
//!
//! Each header can be configured individually if desired. To enable a particular
//! header, call the method for the header on the [`SpaceHelmet`] instance. Multiple
//! method calls can be chained in a builder pattern as illustrated below.
//!
//! ```rust
//! # extern crate rocket;
//! # extern crate rocket_contrib;
//! use rocket::http::uri::Uri;
//!
//! // Every header has a corresponding policy type.
//! use rocket_contrib::space_helmet::{SpaceHelmet, FramePolicy, XssPolicy, HstsPolicy};
//!
//! let site_uri = Uri::parse("https://mysite.example.com").unwrap();
//! let report_uri = Uri::parse("https://report.example.com").unwrap();
//! let helmet = SpaceHelmet::new()
//!     .hsts(HstsPolicy::default()) // each policy has a default.
//!     .no_sniff(None) // setting policy to None disables the header.
//!     .frameguard(FramePolicy::AllowFrom(site_uri))
//!     .xss_protect(XssPolicy::EnableReport(report_uri));
//! ```
//!
//! #### Still have questions?
//!
//! * _What policy should I choose?_ Check out the links in the table
//!  above for individual header documentation. The [helmetjs](https://helmetjs.github.io/) doc's
//!  are also a good resource, and owasp has a collection of [references] on these headers.
//!
//!  [references]: https://www.owasp.org/index.php/OWASP_Secure_Headers_Project#tab=Headers
//!
//! * _Where I can I read more about using fairings?_ Check out the
//! [fairing](https://rocket.rs/guide/fairings/) section of the rocket guide.
//!
//! * _Do I only need those headers `SpaceHelmet` enables by default?_ Maybe, the other headers
//! can protect against many important vulnerabilities. Please consult their documentation and
//! other resources to find out if they are needed for your project.

extern crate time;

mod helmet;
mod policy;

pub use self::helmet::*;
pub use self::policy::*;
