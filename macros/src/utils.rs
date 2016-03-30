use syntax::parse::{token};
use syntax::ast::{Ident, MetaItem, MetaItemKind, LitKind, TokenTree};
use syntax::ext::base::{ExtCtxt};
use syntax::codemap::{Span, Spanned, BytePos, DUMMY_SP};
use syntax::ext::quote::rt::ToTokens;
use syntax::ptr::P;

use std::collections::{HashSet, HashMap};

// macro_rules! debug {
//     ($session:expr, $span:expr, $($message:tt)*) => ({
//         if cfg!(debug) {
//             span_note!($session, $span, "{}:{}", file!(), line!());
//             span_note!($session, $span, $($message)*);
//         }
//     })
// }

macro_rules! debug {
    ($($message:tt)*) => ({
        if DEBUG {
            println!("{}:{}", file!(), line!());
            println!($($message)*);
            println!("");
        }
    })
}

pub fn prepend_ident<T: ToString>(other: T, ident: &Ident) -> Ident {
    let mut new_ident = other.to_string();
    new_ident.push_str(ident.name.to_string().as_str());
    token::str_to_ident(new_ident.as_str())
}

#[allow(dead_code)]
pub fn append_ident<T: ToString>(ident: &Ident, other: T) -> Ident {
    let mut new_ident = ident.name.to_string();
    new_ident.push_str(other.to_string().as_str());
    token::str_to_ident(new_ident.as_str())
}

#[inline]
pub fn wrap_span<T>(t: T, span: Span) -> Spanned<T> {
    Spanned {
        span: span,
        node: t,
    }
}

#[inline]
pub fn dummy_span<T>(t: T) -> Spanned<T> {
    Spanned {
        span: DUMMY_SP,
        node: t,
    }
}

#[inline]
pub fn dummy_kvspan<T>(t: T) -> KVSpanned<T> {
    KVSpanned {
        k_span: DUMMY_SP,
        v_span: DUMMY_SP,
        p_span: DUMMY_SP,
        node: t,
    }
}

#[derive(Debug, Clone)]
pub struct KVSpanned<T> {
    pub k_span: Span,
    pub v_span: Span,
    pub p_span: Span,
    pub node: T
}

impl<T: ToTokens> ToTokens for KVSpanned<T> {
    fn to_tokens(&self, cx: &ExtCtxt) -> Vec<TokenTree> {
        self.node.to_tokens(cx)
    }
}

impl<T> KVSpanned<T> {
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> KVSpanned<U> {
        KVSpanned {
            k_span: self.k_span,
            v_span: self.v_span,
            p_span: self.p_span,
            node: f(self.node),
        }
    }
}

pub fn get_key_values<'b>(ecx: &mut ExtCtxt, sp: Span, required: &[&str],
        optional: &[&str], kv_params: &'b [P<MetaItem>])
            -> HashMap<&'b str, KVSpanned<&'b str>> {
    let mut seen = HashSet::new();
    let mut kv_pairs = HashMap::new();

    // Collect all the kv pairs, keeping track of what we've seen.
    for param in kv_params {
        if let MetaItemKind::NameValue(ref name, ref value) = param.node {
            if required.contains(&&**name) || optional.contains(&&**name) {
                if seen.contains(&**name) {
                    let msg = format!("'{}' parameter appears twice.", &**name);
                    ecx.span_err(param.span, &msg);
                    continue;
                }

                seen.insert(&**name);
                if let LitKind::Str(ref string, _) = value.node {
                    let mut k_span = param.span;
                    k_span.hi = k_span.lo + BytePos(name.len() as u32);
                    kv_pairs.insert(&**name, KVSpanned {
                        node: &**string,
                        k_span: k_span,
                        p_span: param.span,
                        v_span: value.span,
                    });
                } else {
                    ecx.span_err(value.span, "Value must be a string.");
                }
            } else {
                let msg = format!("'{}' is not a valid parameter.", &**name);
                ecx.span_err(param.span, &msg);
            }
        } else {
            ecx.span_err(param.span, "Expected 'key = value', found:");
        }
    }

    // Now, trigger an error for missing `required` params.
    for req in required {
        if !seen.contains(req) {
            let m = format!("'{}' parameter is required but is missing.", req);
            ecx.span_err(sp, &m);
        }
    }

    kv_pairs
}
// pub fn find_value_for(key: &str, kv_params: &[P<MetaItem>]) -> Option<String> {
//     for param in kv_params {
//         if let MetaItemKind::NameValue(ref name, ref value) = param.node {
//             if &**name == key {
//                 if let LitKind::Str(ref string, _) = value.node {
//                     return Some(String::from(&**string));
//                 }
//             }
//         }
//     }

//     None
// }
