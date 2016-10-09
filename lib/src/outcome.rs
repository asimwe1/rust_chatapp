use std::fmt;

use term_painter::Color::*;
use term_painter::ToStyle;

#[must_use]
pub enum Outcome<T> {
    /// Signifies that all processing completed successfully.
    Success,
    /// Signifies that some processing occurred that ultimately resulted in
    /// failure. As a result, no further processing can occur.
    Failure,
    /// Signifies that no processing occured and as such, processing can be
    /// forwarded to the next available target.
    Forward(T),
}

impl<T> Outcome<T> {
    pub fn of<A, B: fmt::Debug>(result: Result<A, B>) -> Outcome<T> {
        if let Err(e) = result {
            error_!("{:?}", e);
            return Outcome::Failure;
        }

        Outcome::Success
    }

    pub fn as_str(&self) -> &'static str {
        match *self {
            Outcome::Success => "Success",
            Outcome::Failure => "FailStop",
            Outcome::Forward(..) => "Forward",
        }
    }

    fn as_int(&self) -> isize {
        match *self {
            Outcome::Success => 0,
            Outcome::Failure => 1,
            Outcome::Forward(..) => 2,
        }
    }

    pub fn expect_success(&self) {
        if *self != Outcome::Success {
            panic!("expected a successful outcome");
        }
    }
}

impl<T> PartialEq for Outcome<T> {
    fn eq(&self, other: &Outcome<T>) -> bool {
        self.as_int() == other.as_int()
    }
}

impl<T> fmt::Debug for Outcome<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Outcome::{}", self.as_str())
    }
}

impl<T> fmt::Display for Outcome<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Outcome::Success => write!(f, "{}", Green.paint("Success")),
            Outcome::Failure => write!(f, "{}", Red.paint("Failure")),
            Outcome::Forward(..) => write!(f, "{}", Cyan.paint("Forwarding")),
        }
    }
}
