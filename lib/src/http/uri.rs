//! Borrowed and owned string types for absolute URIs.
//!

use std::cell::Cell;
use std::convert::From;
use std::fmt::{self, Write};

use router::Collider;

// TODO: Reconsider deriving PartialEq and Eq to make "//a/b" == "/a/b".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct URI<'a> {
    uri: &'a str,
    path: &'a str,
    query: Option<&'a str>,
    fragment: Option<&'a str>,
    segment_count: Cell<Option<usize>>,
}

impl<'a> URI<'a> {
    pub fn new<T: AsRef<str> + ?Sized>(uri: &'a T) -> URI<'a> {
        let uri = uri.as_ref();

        let qmark = uri.find('?');
        let hmark = qmark.map(|i| uri[(i + 1)..].find('#').map(|j| j + i + 1))
            .unwrap_or_else(|| uri.find('#'));

        let (path, query, fragment) = match (qmark, hmark) {
            (Some(i), Some(j)) => (&uri[..i], Some(&uri[(i+1)..j]), Some(&uri[(j+1)..])),
            (Some(i), None) => (&uri[..i], Some(&uri[(i+1)..]), None),
            (None, Some(j)) => (&uri[..j], None, Some(&uri[(j+1)..])),
            (None, None) => (uri, None, None),
        };

        URI {
            segment_count: Cell::new(None),
            uri: uri,
            path: path,
            query: query,
            fragment: fragment,
        }
    }

    #[inline(always)]
    pub fn segment_count(&self) -> usize {
        self.segment_count.get().unwrap_or_else(|| {
            let count = self.segments().count();
            self.segment_count.set(Some(count));
            count
        })
    }

    #[inline(always)]
    pub fn segments(&self) -> Segments<'a> {
        Segments(self.path)
    }

    #[inline(always)]
    pub fn query(&self) -> Option<&'a str> {
        self.query
    }

    #[inline(always)]
    pub fn fragment(&self) -> Option<&'a str> {
        self.fragment
    }

    #[inline(always)]
    pub fn as_str(&self) -> &'a str {
        self.uri
    }
}

impl<'a> fmt::Display for URI<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut last = '\0';
        for c in self.uri.chars() {
            if !(c == '/' && last == '/') {
                f.write_char(c)?;
            }

            last = c;
        }

        Ok(())
    }
}

unsafe impl<'a> Sync for URI<'a> { /* It's safe! */ }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct URIBuf {
    uri: String,
    segment_count: Cell<Option<usize>>,
}

// I don't like repeating all of this stuff. Is there a better way?
impl URIBuf {
    pub fn segment_count(&self) -> usize {
        self.segment_count.get().unwrap_or_else(|| {
            let count = self.segments().count();
            self.segment_count.set(Some(count));
            count
        })
    }

    #[inline(always)]
    pub fn segments(&self) -> Segments {
        self.as_uri_uncached().segments()
    }

    #[inline(always)]
    fn as_uri_uncached(&self) -> URI {
        URI::new(self.uri.as_str())
    }

    #[inline(always)]
    pub fn as_uri(&self) -> URI {
        let mut uri = URI::new(self.uri.as_str());
        uri.segment_count = self.segment_count.clone();
        uri
    }

    #[inline(always)]
    pub fn as_str(&self) -> &str {
        self.uri.as_str()
    }

    #[inline(always)]
    pub fn to_string(&self) -> String {
        self.uri.clone()
    }
}

unsafe impl Sync for URIBuf { /* It's safe! */ }

impl fmt::Display for URIBuf {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_uri_uncached().fmt(f)
    }
}

impl From<String> for URIBuf {
    #[inline(always)]
    fn from(uri: String) -> URIBuf {
        URIBuf {
            segment_count: Cell::new(None),
            uri: uri,
        }
    }
}

impl<'a> From<&'a str> for URIBuf {
    #[inline(always)]
    fn from(uri: &'a str) -> URIBuf {
        URIBuf {
            segment_count: Cell::new(None),
            uri: uri.to_string(),
        }
    }
}

impl<'a, 'b> Collider<URI<'b>> for URI<'a> {
    fn collides_with(&self, other: &URI<'b>) -> bool {
        for (seg_a, seg_b) in self.segments().zip(other.segments()) {
            if seg_a.ends_with("..>") || seg_b.ends_with("..>") {
                return true;
            }

            if !seg_a.collides_with(seg_b) {
                return false;
            }
        }

        if self.segment_count() != other.segment_count() {
            return false;
        }

        true
    }
}

#[derive(Clone, Debug)]
pub struct Segments<'a>(&'a str);

impl<'a> Iterator for Segments<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        // Find the start of the next segment (first that's not '/').
        let i = match self.0.find(|c| c != '/') {
            Some(index) => index,
            None => return None,
        };

        // Get the index of the first character that _is_ a '/' after start.
        // j = index of first character after i (hence the i +) that's not a '/'
        let rest = &self.0[i..];
        let j = rest.find('/').map_or(self.0.len(), |j| i + j);

        // Save the result, update the iterator, and return!
        let result = Some(&self.0[i..j]);
        self.0 = &self.0[j..];
        result
    }

    // TODO: Potentially take a second parameter with Option<cached count> and
    // return it here if it's Some. The downside is that a decision has to be
    // made about -when- to compute and cache that count. A place to do it is in
    // the segments() method. But this means that the count will always be
    // computed regardless of whether it's needed. Maybe this is ok. We'll see.
    // fn count(self) -> usize where Self: Sized {
    //     self.1.unwrap_or_else(self.fold(0, |cnt, _| cnt + 1))
    // }
}

#[cfg(test)]
mod tests {
    use super::{URI, URIBuf};

    fn seg_count(path: &str, expected: usize) -> bool {
        let actual = URI::new(path).segment_count();
        let actual_buf = URIBuf::from(path).segment_count();
        if actual != expected || actual_buf != expected {
            trace_!("Count mismatch: expected {}, got {}.", expected, actual);
            trace_!("{}", if actual != expected { "lifetime" } else { "buf" });
            trace_!("Segments (for {}):", path);
            for (i, segment) in URI::new(path).segments().enumerate() {
                trace_!("{}: {}", i, segment);
            }
        }

        actual == expected && actual_buf == expected
    }

    fn eq_segments(path: &str, expected: &[&str]) -> bool {
        let uri = URI::new(path);
        let actual: Vec<&str> = uri.segments().collect();

        let uri_buf = URIBuf::from(path);
        let actual_buf: Vec<&str> = uri_buf.segments().collect();

        actual == expected && actual_buf == expected
    }

    #[test]
    fn simple_segment_count() {
        assert!(seg_count("", 0));
        assert!(seg_count("/", 0));
        assert!(seg_count("a", 1));
        assert!(seg_count("/a", 1));
        assert!(seg_count("a/", 1));
        assert!(seg_count("/a/", 1));
        assert!(seg_count("/a/b", 2));
        assert!(seg_count("/a/b/", 2));
        assert!(seg_count("a/b/", 2));
        assert!(seg_count("ab/", 1));
    }

    #[test]
    fn segment_count() {
        assert!(seg_count("////", 0));
        assert!(seg_count("//a//", 1));
        assert!(seg_count("//abc//", 1));
        assert!(seg_count("//abc/def/", 2));
        assert!(seg_count("//////abc///def//////////", 2));
        assert!(seg_count("a/b/c/d/e/f/g", 7));
        assert!(seg_count("/a/b/c/d/e/f/g", 7));
        assert!(seg_count("/a/b/c/d/e/f/g/", 7));
        assert!(seg_count("/a/b/cdjflk/d/e/f/g", 7));
        assert!(seg_count("//aaflja/b/cdjflk/d/e/f/g", 7));
        assert!(seg_count("/a   /b", 2));
    }

    #[test]
    fn single_segments_match() {
        assert!(eq_segments("", &[]));
        assert!(eq_segments("a", &["a"]));
        assert!(eq_segments("/a", &["a"]));
        assert!(eq_segments("/a/", &["a"]));
        assert!(eq_segments("a/", &["a"]));
        assert!(eq_segments("///a/", &["a"]));
        assert!(eq_segments("///a///////", &["a"]));
        assert!(eq_segments("a///////", &["a"]));
        assert!(eq_segments("//a", &["a"]));
        assert!(eq_segments("", &[]));
        assert!(eq_segments("abc", &["abc"]));
        assert!(eq_segments("/a", &["a"]));
        assert!(eq_segments("/abc/", &["abc"]));
        assert!(eq_segments("abc/", &["abc"]));
        assert!(eq_segments("///abc/", &["abc"]));
        assert!(eq_segments("///abc///////", &["abc"]));
        assert!(eq_segments("abc///////", &["abc"]));
        assert!(eq_segments("//abc", &["abc"]));
    }

    #[test]
    fn multi_segments_match() {
        assert!(eq_segments("a/b/c", &["a", "b", "c"]));
        assert!(eq_segments("/a/b", &["a", "b"]));
        assert!(eq_segments("/a///b", &["a", "b"]));
        assert!(eq_segments("a/b/c/d", &["a", "b", "c", "d"]));
        assert!(eq_segments("///a///////d////c", &["a", "d", "c"]));
        assert!(eq_segments("abc/abc", &["abc", "abc"]));
        assert!(eq_segments("abc/abc/", &["abc", "abc"]));
        assert!(eq_segments("///abc///////a", &["abc", "a"]));
        assert!(eq_segments("/////abc/b", &["abc", "b"]));
        assert!(eq_segments("//abc//c////////d", &["abc", "c", "d"]));
    }

    #[test]
    fn multi_segments_match_funky_chars() {
        assert!(eq_segments("a/b/c!!!", &["a", "b", "c!!!"]));
        assert!(eq_segments("a  /b", &["a  ", "b"]));
        assert!(eq_segments("  a/b", &["  a", "b"]));
        assert!(eq_segments("  a/b  ", &["  a", "b  "]));
        assert!(eq_segments("  a///b  ", &["  a", "b  "]));
        assert!(eq_segments("  ab  ", &["  ab  "]));
    }

    #[test]
    fn segment_mismatch() {
        assert!(!eq_segments("", &["a"]));
        assert!(!eq_segments("a", &[]));
        assert!(!eq_segments("/a/a", &["a"]));
        assert!(!eq_segments("/a/b", &["b", "a"]));
        assert!(!eq_segments("/a/a/b", &["a", "b"]));
        assert!(!eq_segments("///a/", &[]));
    }

    fn test_query(uri: &str, query: Option<&str>) {
        let uri = URI::new(uri);
        assert_eq!(uri.query(), query);
    }

    fn test_fragment(uri: &str, fragment: Option<&str>) {
        let uri = URI::new(uri);
        assert_eq!(uri.fragment(), fragment);
    }

    #[test]
    fn query_does_not_exist() {
        test_query("/test", None);
        test_query("/a/b/c/d/e", None);
        test_query("/////", None);
        test_query("//a///", None);
    }

    #[test]
    fn query_exists() {
        test_query("/test?abc", Some("abc"));
        test_query("/a/b/c?abc", Some("abc"));
        test_query("/a/b/c/d/e/f/g/?abc#hijklmnop", Some("abc"));
        test_query("?123", Some("123"));
        test_query("?", Some(""));
        test_query("/?", Some(""));
        test_query("?#", Some(""));
        test_query("/?hi", Some("hi"));
    }

    #[test]
    fn fragment_exists() {
        test_fragment("/test#abc", Some("abc"));
        test_fragment("/#abc", Some("abc"));
        test_fragment("/a/b/c?123#a", Some("a"));
        test_fragment("/#a", Some("a"));
    }

    #[test]
    fn fragment_does_not_exist() {
        test_fragment("/testabc", None);
        test_fragment("/abc", None);
        test_fragment("/a/b/c?123", None);
        test_fragment("/a", None);
    }
}
