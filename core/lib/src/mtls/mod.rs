//! Support for mutual TLS client certificates.
//!
//! For details on how to configure mutual TLS, see
//! [`MutualTls`](crate::config::MutualTls) and the [TLS
//! guide](https://rocket.rs/master/guide/configuration/#tls). See
//! [`Certificate`] for a request guard that validated, verifies, and retrieves
//! client certificates.

pub mod oid {
    //! Lower-level OID types re-exported from
    //! [`oid_registry`](https://docs.rs/oid-registry/0.4) and
    //! [`der-parser`](https://docs.rs/der-parser/7).

    pub use x509_parser::oid_registry::*;
    pub use x509_parser::objects::*;
}

pub mod bigint {
    //! Signed and unsigned big integer types re-exported from
    //! [`num_bigint`](https://docs.rs/num-bigint/0.4).
    pub use x509_parser::der_parser::num_bigint::*;
}

pub mod x509 {
    //! Lower-level X.509 types re-exported from
    //! [`x509_parser`](https://docs.rs/x509-parser/0.13).
    //!
    //! Lack of documentation is directly inherited from the source crate.
    //! Prefer to use Rocket's wrappers when possible.

    pub(crate) use x509_parser::nom;
    pub use x509_parser::certificate::*;
    pub use x509_parser::cri_attributes::*;
    pub use x509_parser::error::*;
    pub use x509_parser::extensions::*;
    pub use x509_parser::revocation_list::*;
    pub use x509_parser::time::*;
    pub use x509_parser::x509::*;
    pub use x509_parser::der_parser::der;
    pub use x509_parser::der_parser::ber;
    pub use x509_parser::traits::*;
}

mod certificate;
mod error;
mod name;
mod config;

pub use error::Error;
pub use name::Name;
pub use config::MtlsConfig;
pub use certificate::{Certificate, CertificateDer};

/// A type alias for [`Result`](std::result::Result) with the error type set to
/// [`Error`].
pub type Result<T, E = Error> = std::result::Result<T, E>;
