use std::ops::BitOr;

/// Information about a [`Fairing`](/rocket/fairing/trait.Fairing.html).
///
/// The `name` field is an arbitrary name for a fairing. The `kind` field is a
/// is an `or`d set of [`Kind`](/rocket/fairing/struct.Kind.html) structures.
/// Rocket uses the values set in `Kind` to determine which callbacks from a
/// given `Fairing` implementation to actually call.
///
/// # Example
///
/// A simple `Info` structure that can be used for a `Fairing` that implements
/// all three callbacks:
///
/// ```
/// use rocket::fairing::{Info, Kind};
///
/// # let _unused_info =
/// Info {
///     name: "Example Fairing",
///     kind: Kind::Launch | Kind::Request | Kind::Response
/// }
/// # ;
/// ```
pub struct Info {
    /// The name of the fairing.
    pub name: &'static str,
    /// A set representing the callbacks the fairing wishes to receive.
    pub kind: Kind
}

/// A bitset representing the kinds of callbacks a
/// [`Fairing`](/rocket/fairing/trait.Fairing.html) wishes to receive.
///
/// A fairing can request any combination of any of the following kinds of
/// callbacks:
///
///   * Launch
///   * Request
///   * Response
///
/// Two `Kind` structures can be `or`d together to represent a combination. For
/// instance, to represent a fairing that is both a launch and request fairing,
/// use `Kind::Launch | Kind::Request`. Similarly, to represent a fairing that
/// is all three kinds, use `Kind::Launch | Kind::Request | Kind::Response`.
#[derive(Debug, Clone, Copy)]
pub struct Kind(usize);

#[allow(non_upper_case_globals)]
impl Kind {
    /// `Kind` flag representing a request for a 'launch' callback.
    pub const Launch: Kind = Kind(0b001);
    /// `Kind` flag representing a request for a 'request' callback.
    pub const Request: Kind = Kind(0b010);
    /// `Kind` flag representing a request for a 'response' callback.
    pub const Response: Kind = Kind(0b100);

    /// Returns `true` if `self` is a superset of `other`. In other words,
    /// returns `true` if all of the kinds in `other` are also in `self`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket::fairing::Kind;
    ///
    /// let launch_and_req = Kind::Launch | Kind::Request;
    /// assert!(launch_and_req.is(Kind::Launch | Kind::Request));
    ///
    /// assert!(launch_and_req.is(Kind::Launch));
    /// assert!(launch_and_req.is(Kind::Request));
    ///
    /// assert!(!launch_and_req.is(Kind::Response));
    /// assert!(!launch_and_req.is(Kind::Launch | Kind::Response));
    /// assert!(!launch_and_req.is(Kind::Launch | Kind::Request | Kind::Response));
    /// ```
    #[inline]
    pub fn is(self, other: Kind) -> bool {
        (other.0 & self.0) == other.0
    }
}

impl BitOr for Kind {
    type Output = Self;

    #[inline(always)]
    fn bitor(self, rhs: Self) -> Self {
        Kind(self.0 | rhs.0)
    }
}
