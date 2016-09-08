use syntax::ast::Ident;
use syntax::ext::base::ExtCtxt;
use syntax::codemap::{Span, Spanned, BytePos};
use syntax::parse::token::str_to_ident;

use utils::{span, SpanExt};

#[derive(Debug)]
pub enum Param {
    Single(Spanned<Ident>),
    Many(Spanned<Ident>)
}

impl Param {
    pub fn inner(&self) -> &Spanned<Ident> {
        match *self {
            Param::Single(ref ident) | Param::Many(ref ident) => ident
        }
    }

    pub fn ident(&self) -> &Ident {
        match *self {
            Param::Single(ref ident) | Param::Many(ref ident) => &ident.node
        }
    }
}

pub struct ParamIter<'s, 'a, 'c: 'a> {
    ctxt: &'a ExtCtxt<'c>,
    span: Span,
    string: &'s str,
}

impl<'s, 'a, 'c: 'a> ParamIter<'s, 'a, 'c> {
    pub fn new(c: &'a ExtCtxt<'c>, s: &'s str, p: Span) -> ParamIter<'s, 'a, 'c> {
        ParamIter {
            ctxt: c,
            span: p,
            string: s,
        }
    }
}

impl<'s, 'a, 'c> Iterator for ParamIter<'s, 'a, 'c> {
    type Item = Param;

    fn next(&mut self) -> Option<Param> {
        // Find the start and end indexes for the next parameter, if any.
        let (start, end) = match (self.string.find('<'), self.string.find('>')) {
            (Some(i), Some(j)) => (i, j),
            _ => return None,
        };

        // Ensure we found a valid parameter.
        if end <= start {
            self.ctxt.span_err(self.span, "Parameter list is malformed.");
            return None;
        }

        // Calculate the parameter's ident.
        let full_param = &self.string[(start + 1)..end];
        let (is_many, param) = match full_param.ends_with("..") {
            true => (true, &full_param[..(full_param.len() - 2)]),
            false => (false, full_param)
        };

        let mut param_span = self.span;
        param_span.lo = self.span.lo + BytePos(start as u32);
        param_span.hi = self.span.lo + BytePos((end + 1) as u32);

        // Advance the string and span.
        self.string = &self.string[(end + 1)..];
        self.span.lo = self.span.lo + BytePos((end + 1) as u32);

        // Check for nonemptiness, that the characters are correct, and return.
        if param.is_empty() {
            self.ctxt.span_err(param_span, "parameter names cannot be empty");
            None
        } else if param.contains(|c: char| !c.is_alphanumeric()) {
            self.ctxt.span_err(param_span, "parameter names must be alphanumeric");
            None
        } else if is_many && !self.string.is_empty() {
            let sp = self.span.shorten_to(self.string.len() as u32);
            self.ctxt.struct_span_err(sp, "text after a trailing '..' param")
                     .span_note(param_span, "trailing param is here")
                     .emit();
            None
        } else {
            let spanned_ident = span(str_to_ident(param), param_span);
            match is_many {
                true => Some(Param::Many(spanned_ident)),
                false => Some(Param::Single(spanned_ident))
            }
        }

    }
}
