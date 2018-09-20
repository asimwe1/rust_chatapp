use std::hash::{Hash, Hasher};

use derive_utils::syn;
use proc_macro::{Span, Diagnostic};

use http::route::RouteSegment;
use proc_macro_ext::{SpanExt, Diagnostics, PResult, DResult};

pub use http::route::{Error, Kind, Source};

#[derive(Debug, Clone)]
pub struct Segment {
    pub span: Span,
    pub kind: Kind,
    pub source: Source,
    pub name: String,
    pub index: Option<usize>,
}

impl Segment {
    fn from(segment: RouteSegment, span: Span) -> Segment {
        let (kind, source, index) = (segment.kind, segment.source, segment.index);
        Segment { span, kind, source, index, name: segment.name.into_owned() }
    }

    crate fn to_route_segment<'a>(&'a self) -> String {
        match (self.source, self.kind) {
            (_, Kind::Single) => format!("<{}>", self.name),
            (_, Kind::Multi) => format!("<{}..>", self.name),
            (_, Kind::Static) => self.name.clone()
        }
    }
}

impl<'a> From<&'a syn::Ident> for Segment {
    fn from(ident: &syn::Ident) -> Segment {
        Segment {
            kind: Kind::Static,
            source: Source::Unknown,
            span: ident.span().unstable(),
            name: ident.to_string(),
            index: None,
        }
    }
}

impl PartialEq for Segment {
    fn eq(&self, other: &Segment) -> bool {
        self.name == other.name
    }
}

impl Eq for Segment {  }

impl Hash for Segment {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

pub fn subspan(needle: &str, haystack: &str, span: Span) -> Option<Span> {
    let index = needle.as_ptr() as usize - haystack.as_ptr() as usize;
    let remaining = haystack.len() - (index + needle.len());
    span.trimmed(index, remaining)
}

pub fn trailspan(needle: &str, haystack: &str, span: Span) -> Option<Span> {
    let index = needle.as_ptr() as usize - haystack.as_ptr() as usize;
    span.trimmed(index - 1, 0)
}

pub fn into_diagnostic(
    segment: &str, // The segment that failed.
    source: &str,  // The haystack where `segment` can be found.
    span: Span,    // The `Span` of `Source`.
    error: &Error,  // The error.
) -> Diagnostic {
    let seg_span = subspan(segment, source, span).unwrap();
    match error {
        Error::Empty => {
            seg_span.error("parameter names cannot be empty")
        }
        Error::Ident(name) => {
            seg_span.error(format!("`{}` is not a valid identifier", name))
                .help("parameter names must be valid identifiers")
        }
        Error::Ignored => {
            seg_span.error("parameters must be named")
                .help("use a name such as `_guard` or `_param`")
        }
        Error::MissingClose => {
            seg_span.error("parameter is missing a closing bracket")
                .help(format!("did you mean '{}>'?", segment))
        }
        Error::Malformed => {
            seg_span.error("malformed parameter or identifier")
                .help("parameters must be of the form '<param>'")
                .help("identifiers cannot contain '<' or '>'")
        }
        Error::Uri => {
            seg_span.error("component contains invalid URI characters")
                .note("components cannot contain '%' and '+' characters")
        }
        Error::Trailing(multi) => {
            let multi_span = subspan(multi, source, span).unwrap();
            trailspan(segment, source, span).unwrap()
                .error("unexpected trailing text after a '..' param")
                .help("a multi-segment param must be the final component")
                .span_note(multi_span, "multi-segment param is here")
        }
    }
}

pub fn parse_segment(segment: &str, span: Span) -> PResult<Segment> {
    RouteSegment::parse_one(segment)
        .map(|segment| Segment::from(segment, span))
        .map_err(|e| into_diagnostic(segment, segment, span, &e))
}

pub fn parse_segments(
    string: &str,
    sep: char,
    source: Source,
    span: Span
) -> DResult<Vec<Segment>> {
    let mut segments = vec![];
    let mut diags = Diagnostics::new();

    for result in RouteSegment::parse_many(string, sep, source) {
        if let Err((segment_string, error)) = result {
            diags.push(into_diagnostic(segment_string, string, span, &error));
            if let Error::Trailing(..) = error {
                break;
            }
        } else if let Ok(segment) = result {
            let seg_span = subspan(&segment.string, string, span).unwrap();
            segments.push(Segment::from(segment, seg_span));
        }
    }

    diags.err_or(segments)
}
