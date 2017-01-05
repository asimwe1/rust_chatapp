//! Types and traits for reading and parsing request body data.

#[cfg(any(test, feature = "testing"))]
#[path = "."]
mod items {
    mod test_data;

    pub use self::test_data::Data;
    pub use self::test_data::DataStream;
}

#[cfg(not(any(test, feature = "testing")))]
#[path = "."]
mod items {
    mod data;
    mod data_stream;

    pub use self::data::Data;
    pub use self::data_stream::DataStream;
}

mod from_data;

pub use self::from_data::{FromData, Outcome};
pub use self::items::{Data, DataStream};
