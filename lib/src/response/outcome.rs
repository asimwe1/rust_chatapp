use response::*;

use term_painter::Color::*;
use term_painter::ToStyle;
use std::fmt;

pub enum Outcome<'h> {
    Complete,
    FailStop,
    FailForward(HyperResponse<'h, HyperFresh>),
}

impl<'h> Outcome<'h> {
    pub fn as_str(&self) -> &'static str {
        match self {
            &Outcome::Complete => "Complete",
            &Outcome::FailStop => "FailStop",
            &Outcome::FailForward(..) => "FailForward",
        }
    }

    pub fn is_forward(&self) -> bool {
        match self {
            &Outcome::FailForward(_) => true,
            _ => false
        }
    }

    pub fn map_forward<F>(self, f: F)
            where F: FnOnce(FreshHyperResponse<'h>) {
        match self {
            Outcome::FailForward(res) => f(res),
            _ => { /* nothing */ }
        }
    }

    pub fn map_forward_or<F, R>(self, default: R, f: F) -> R
            where F: FnOnce(FreshHyperResponse<'h>) -> R {
        match self {
            Outcome::FailForward(res) => f(res),
            _ => default
        }
    }

    pub fn is_failure(&self) -> bool {
        self == &Outcome::FailStop
    }

    pub fn is_complete(&self) -> bool {
        self == &Outcome::Complete
    }

    fn as_int(&self) -> isize {
        match self {
            &Outcome::Complete => 0,
            &Outcome::FailStop => 1,
            &Outcome::FailForward(..) => 2,
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
        match self {
            &Outcome::Complete => {
                write!(f, "{}", Green.paint("Complete"))
            },
            &Outcome::FailStop => {
                write!(f, "{}", Red.paint("Failed"))
            },
            &Outcome::FailForward(..) => {
                write!(f, "{}", Yellow.paint("Forwarding"))
            },
        }
    }
}
