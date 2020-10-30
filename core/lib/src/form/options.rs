#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Options {
    pub strict: bool,
}

#[allow(non_upper_case_globals, dead_code)]
impl Options {
    pub const Lenient: Self = Options { strict: false };

    pub const Strict: Self = Options { strict: true };
}
