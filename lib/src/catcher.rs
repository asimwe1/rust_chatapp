use handler::Handler;
use codegen::StaticCatchInfo;

use std::fmt;
use term_painter::ToStyle;
use term_painter::Color::*;

pub struct Catcher {
    pub code: u16,
    pub handler: Handler,
}

impl Catcher {
    pub fn new(code: u16, handler: Handler) -> Catcher {
        Catcher {
            code: code,
            handler: handler,
        }
    }
}

impl<'a> From<&'a StaticCatchInfo> for Catcher {
    fn from(info: &'a StaticCatchInfo) -> Catcher {
        Catcher::new(info.code, info.handler)
    }
}

impl fmt::Display for Catcher {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", Blue.paint(&self.code), Blue.paint("catcher."))
    }
}
