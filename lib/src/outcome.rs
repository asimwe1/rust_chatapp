use std::fmt;

use term_painter::Color::*;
use term_painter::Color;
use term_painter::ToStyle;

#[must_use]
pub enum Outcome<S, E, F> {
    /// Contains the success value.
    Success(S),
    /// Contains the failure error value.
    Failure(E),
    /// Contains the value to forward on.
    Forward(F),
}

impl<S, E, F> Outcome<S, E, F> {
    /// Unwraps the Outcome, yielding the contents of a Success.
    ///
    /// # Panics
    ///
    /// Panics if the value is not Success.
    #[inline(always)]
    pub fn unwrap(self) -> S {
        match self {
            Outcome::Success(val) => val,
            _ => panic!("Expected a successful outcome!")
        }
    }

    /// Return true if this `Outcome` is a `Success`.
    #[inline(always)]
    pub fn is_success(&self) -> bool {
        match *self {
            Outcome::Success(_) => true,
            _ => false
        }
    }

    /// Return true if this `Outcome` is a `Failure`.
    #[inline(always)]
    pub fn is_failure(&self) -> bool {
        match *self {
            Outcome::Failure(_) => true,
            _ => false
        }
    }

    /// Return true if this `Outcome` is a `Forward`.
    #[inline(always)]
    pub fn is_forward(&self) -> bool {
        match *self {
            Outcome::Forward(_) => true,
            _ => false
        }
    }

    /// Converts from `Outcome<S, E, F>` to `Option<S>`.
    ///
    /// Returns the `Some` of the `Success` if this is a `Success`, otherwise
    /// returns `None`. `self` is consumed, and all other values are discarded.
    #[inline(always)]
    pub fn succeeded(self) -> Option<S> {
        match self {
            Outcome::Success(val) => Some(val),
            _ => None
        }
    }

    /// Converts from `Outcome<S, E, F>` to `Option<E>`.
    ///
    /// Returns the `Some` of the `Failure` if this is a `Failure`, otherwise
    /// returns `None`. `self` is consumed, and all other values are discarded.
    #[inline(always)]
    pub fn failed(self) -> Option<E> {
        match self {
            Outcome::Failure(val) => Some(val),
            _ => None
        }
    }

    /// Converts from `Outcome<S, E, F>` to `Option<F>`.
    ///
    /// Returns the `Some` of the `Forward` if this is a `Forward`, otherwise
    /// returns `None`. `self` is consumed, and all other values are discarded.
    #[inline(always)]
    pub fn forwarded(self) -> Option<F> {
        match self {
            Outcome::Forward(val) => Some(val),
            _ => None
        }
    }

    #[inline(always)]
    fn formatting(&self) -> (Color, &'static str) {
        match *self {
            Outcome::Success(..) => (Green, "Succcess"),
            Outcome::Failure(..) => (Red, "Failure"),
            Outcome::Forward(..) => (Yellow, "Forward"),
        }
    }
}

impl<S, E, F> fmt::Debug for Outcome<S, E, F> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Outcome::{}", self.formatting().1)
    }
}

impl<S, E, F> fmt::Display for Outcome<S, E, F> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (color, string) = self.formatting();
        write!(f, "{}", color.paint(string))
    }
}
