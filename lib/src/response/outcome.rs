use std::fmt;

use term_painter::Color::*;
use term_painter::ToStyle;

use http::hyper::FreshHyperResponse;

pub enum Outcome<T> {
    /// Signifies a response that completed sucessfully.
    Success,
    /// Signifies a failing response that started responding but fail, so no
    /// further processing can occur.
    FailStop,
    /// Signifies a response that failed internally without beginning to
    /// respond but no further processing should occur.
    Bad(T),
    /// Signifies a failing response that failed internally without beginning to
    /// respond. Further processing should be attempted.
    FailForward(T),
}

pub type ResponseOutcome<'a> = Outcome<FreshHyperResponse<'a>>;

impl<T> Outcome<T> {
    pub fn as_str(&self) -> &'static str {
        match *self {
            Outcome::Success => "Success",
            Outcome::FailStop => "FailStop",
            Outcome::Bad(..) => "Bad",
            Outcome::FailForward(..) => "FailForward",
        }
    }

    fn as_int(&self) -> isize {
        match *self {
            Outcome::Success => 0,
            Outcome::Bad(..) => 1,
            Outcome::FailStop => 2,
            Outcome::FailForward(..) => 3,
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
            Outcome::Bad(..) => write!(f, "{}", Yellow.paint("Bad Completion")),
            Outcome::FailStop => write!(f, "{}", Red.paint("Failed")),
            Outcome::FailForward(..) => write!(f, "{}", Cyan.paint("Forwarding")),
        }
    }
}
