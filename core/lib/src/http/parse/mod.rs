mod media_type;
mod accept;
mod indexed;
mod checkers;

pub use self::indexed::*;
pub use self::media_type::*;
pub use self::accept::*;

pub type Input<'a> = IndexedInput<'a, str>;
pub type Slice<'a> = Indexed<'a, str>;
pub type Result<'a, T> = ::pear::Result<T, Input<'a>>;
