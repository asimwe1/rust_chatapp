use syntax::codemap::{Span, BytePos};

pub trait SpanExt {
    fn shorten_to(self, to_length: u32) -> Span;
}

impl SpanExt for Span {
    fn shorten_to(mut self, to_length: u32) -> Span {
        self.hi = self.lo + BytePos(to_length);
        self
    }
}
