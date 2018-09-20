use proc_macro::{Span, Diagnostic, /* MultiSpan */};
use syntax_pos::{Span as InnerSpan, Pos, BytePos};

pub type PResult<T> = ::std::result::Result<T, Diagnostic>;

pub type DResult<T> = ::std::result::Result<T, Diagnostics>;

// An experiment.
pub struct Diagnostics(Vec<Diagnostic>);

impl Diagnostics {
    pub fn new() -> Self {
        Diagnostics(vec![])
    }

    pub fn push(&mut self, diag: Diagnostic) {
        self.0.push(diag);
    }

    pub fn join(mut self, mut diags: Diagnostics) -> Self {
        self.0.append(&mut diags.0);
        self
    }

    pub fn emit_head(self) -> Diagnostic {
        let mut iter = self.0.into_iter();
        let mut last = iter.next().expect("Diagnostic::emit_head empty");
        for diag in iter {
            last.emit();
            last = diag;
        }

        last
    }

    pub fn head_err_or<T>(self, ok: T) -> PResult<T> {
        match self.0.is_empty() {
            true => Ok(ok),
            false => Err(self.emit_head())
        }
    }

    pub fn err_or<T>(self, ok: T) -> DResult<T> {
        match self.0.is_empty() {
            true => Ok(ok),
            false => Err(self)
        }
    }
}

impl From<Diagnostic> for Diagnostics {
    fn from(diag: Diagnostic) -> Self {
        Diagnostics(vec![diag])
    }
}

impl From<Vec<Diagnostic>> for Diagnostics {
    fn from(diags: Vec<Diagnostic>) -> Self {
        Diagnostics(diags)
    }
}

pub trait SpanExt {
    fn trimmed(&self, left: usize, right: usize) -> Option<Span>;
}

impl SpanExt for Span {
    /// Trim the span on the left by `left` characters and on the right by
    /// `right` characters.
    fn trimmed(&self, left: usize, right: usize) -> Option<Span> {
        let inner: InnerSpan = unsafe { ::std::mem::transmute(*self) };
        if left > u32::max_value() as usize || right > u32::max_value() as usize {
            return None;
        }

        // Ensure that the addition won't overflow.
        let (left, right) = (left as u32, right as u32);
        if u32::max_value() - left < inner.lo().to_u32() {
            return None;
        }

        // Ensure that the subtraction won't underflow.
        if right > inner.hi().to_u32() {
            return None;
        }

        let new_lo = inner.lo() + BytePos(left);
        let new_hi = inner.hi() - BytePos(right);

        // Ensure we're still inside the old `Span` and didn't cross paths.
        if new_lo >= new_hi {
            return None;
        }

        let new_inner = inner.with_lo(new_lo).with_hi(new_hi);
        Some(unsafe { ::std::mem::transmute(new_inner) })
    }
}
