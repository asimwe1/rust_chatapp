use syntax::ast::Ident;
use syntax::ext::base::ExtCtxt;
use syntax::codemap::{Span, Spanned, BytePos};
use syntax::parse::token::str_to_ident;

use utils::span;

pub struct ParamIter<'s, 'a, 'c: 'a> {
    ctxt: &'a ExtCtxt<'c>,
    span: Span,
    string: &'s str
}

impl<'s, 'a, 'c: 'a> ParamIter<'s, 'a, 'c> {
    pub fn new(c: &'a ExtCtxt<'c>, s: &'s str, p: Span) -> ParamIter<'s, 'a, 'c> {
        ParamIter {
            ctxt: c,
            span: p,
            string: s
        }
    }
}

impl<'s, 'a, 'c> Iterator for ParamIter<'s, 'a, 'c> {
    type Item = Spanned<Ident>;

    fn next(&mut self) -> Option<Spanned<Ident>> {
        // Find the start and end indexes for the next parameter, if any.
        let (start, end) = match (self.string.find('<'), self.string.find('>')) {
            (Some(i), Some(j)) => (i, j),
            _ => return None
        };

        // Ensure we found a valid parameter.
        if end <= start {
            self.ctxt.span_err(self.span, "Parameter list is malformed.");
            return None;
        }

        // Calculate the parameter and the span for the parameter.
        let param = &self.string[(start + 1)..end];
        let mut param_span = self.span;
        param_span.lo = self.span.lo + BytePos(start as u32);
        param_span.hi = self.span.lo + BytePos((end + 1) as u32);

        // Check for nonemptiness and that the characters are correct.
        if param.is_empty() {
            self.ctxt.span_err(param_span, "parameter names cannot be empty");
            None
        } else if param.contains(|c: char| !c.is_alphanumeric()) {
            self.ctxt.span_err(param_span, "parameters must be alphanumeric");
            None
        } else {
            self.string = &self.string[(end + 1)..];
            self.span.lo = self.span.lo + BytePos((end + 1) as u32);
            Some(span(str_to_ident(param), param_span))
        }
    }
}

