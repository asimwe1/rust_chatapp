//! Types and traits for reading and parsing request body data.

#[cfg(any(test, feature = "testing"))] mod test_data;
#[cfg(not(any(test, feature = "testing")))] mod data;
#[cfg(not(any(test, feature = "testing")))] mod data_stream;
mod from_data;

pub use self::from_data::{FromData, Outcome};

#[cfg(any(test, feature = "testing"))] pub use self::test_data::Data;
#[cfg(not(any(test, feature = "testing")))] pub use self::data::Data;
