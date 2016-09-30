use response::*;

use term_painter::Color::*;
use term_painter::ToStyle;
use std::fmt;

pub enum Outcome<'h> {
    /// Signifies a response that completed sucessfully.
    Complete,
    /// Signifies a response that failed internally.
    Bad(FreshHyperResponse<'h>),
    /// Signifies a failing response where no further processing should happen.
    FailStop,
    /// Signifies a failing response whose request should be processed further.
    FailForward(FreshHyperResponse<'h>),
}

impl<'h> Outcome<'h> {
    pub fn as_str(&self) -> &'static str {
        match *self {
            Outcome::Complete => "Complete",
            Outcome::FailStop => "FailStop",
            Outcome::Bad(..) => "Bad",
            Outcome::FailForward(..) => "FailForward",
        }
    }

    fn as_int(&self) -> isize {
        match *self {
            Outcome::Complete => 0,
            Outcome::Bad(..) => 1,
            Outcome::FailStop => 2,
            Outcome::FailForward(..) => 3,
        }
    }
}

impl<'h> PartialEq for Outcome<'h> {
    fn eq(&self, other: &Outcome<'h>) -> bool {
        self.as_int() == other.as_int()
    }
}

impl<'h> fmt::Debug for Outcome<'h> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Outcome::{}", self.as_str())
    }
}

impl<'h> fmt::Display for Outcome<'h> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Outcome::Complete => {
                write!(f, "{}", Green.paint("Complete"))
            },
            Outcome::Bad(..) => {
                write!(f, "{}", Yellow.paint("Bad Completion"))
            },
            Outcome::FailStop => {
                write!(f, "{}", Red.paint("Failed"))
            },
            Outcome::FailForward(..) => {
                write!(f, "{}", Cyan.paint("Forwarding"))
            },
        }
    }
}
