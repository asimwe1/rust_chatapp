use std::str::FromStr;

use syntax::ext::base::{Annotatable, ExtCtxt};
use syntax::ast::{Item, ItemKind, MetaItem, FnDecl};
use syntax::codemap::{Span, Spanned, BytePos};
use syntax::ptr::P;

use utils::*;
use rocket::Method;

pub struct MetaItemParser<'a, 'c: 'a> {
    attr_name: &'a str,
    ctxt: &'a ExtCtxt<'c>,
    meta_item: &'a MetaItem,
    annotated: &'a Annotatable,
    span: Span
}

pub struct ParamIter<'s, 'a, 'c: 'a> {
    ctxt: &'a ExtCtxt<'c>,
    span: Span,
    string: &'s str
}

impl<'a, 'c> MetaItemParser<'a, 'c> {
    pub fn new(ctxt: &'a ExtCtxt<'c>, meta_item: &'a MetaItem,
               annotated: &'a Annotatable, span: &'a Span) -> MetaItemParser<'a, 'c> {
        MetaItemParser {
            attr_name: meta_item.name(),
            ctxt: ctxt,
            meta_item: meta_item,
            annotated: annotated,
            span: span.clone(),
        }
    }

    fn bad_item(&self, expected: &str, got: &str, sp: Span) -> ! {
        let msg_a = format!("Expected a {} item...", expected);
        let msg_b = format!("...but found a {} item instead.", got);
        self.ctxt.span_err(self.span, msg_a.as_str());
        self.ctxt.span_fatal(sp, msg_b.as_str())
    }


    pub fn expect_item(&self) -> &'a P<Item> {
        let bad_item = |name: &str, sp: Span| self.bad_item("regular", name, sp);

        match *self.annotated {
            Annotatable::Item(ref item) => item,
            Annotatable::TraitItem(ref item) => bad_item("trait", item.span),
            Annotatable::ImplItem(ref item) => bad_item("impl", item.span)
        }
    }

    pub fn expect_fn_decl(&self) -> Spanned<&'a FnDecl> {
        let item = self.expect_item();
        let bad_item = |name: &str| self.bad_item("fn_decl", name, item.span);

        let fn_decl: &P<FnDecl> = match item.node {
             ItemKind::Fn(ref decl, _, _, _, _, _) => decl,
             _ => bad_item("other")
         };

        span(fn_decl, item.span)
    }

    fn expect_list(&self) -> &'a Vec<P<MetaItem>> {
        let msg = format!("Bad use. Expected: #[{}(...)]", self.attr_name);
        self.meta_item.expect_list(self.ctxt, msg.as_str())
    }

    pub fn iter_params<'s>(&self, from: &Spanned<&'s str>) -> ParamIter<'s, 'a, 'c> {
        ParamIter {
            ctxt: self.ctxt,
            span: from.span,
            string: from.node
        }
    }
}

impl<'s, 'a, 'c> Iterator for ParamIter<'s, 'a, 'c> {
    type Item = Spanned<&'s str>;

    fn next(&mut self) -> Option<Spanned<&'s str>> {
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
        if param.len() == 0 {
            self.ctxt.span_err(param_span, "Parameter names cannot be empty.");
            None
        } else if param.contains(|c: char| !c.is_alphanumeric()) {
            self.ctxt.span_err(param_span, "Parameters must be alphanumeric.");
            None
        } else {
            self.string = &self.string[(end + 1)..];
            self.span.lo = self.span.lo + BytePos((end + 1) as u32);
            Some(span(param, param_span))
        }
    }
}

pub struct RouteParams {
    pub method: Spanned<Method>,
    pub path: KVSpanned<String>,
    pub form: Option<KVSpanned<String>>,
}

pub trait RouteDecoratorExt {
    fn bad_method(&self, sp: Span, message: &str);
    fn parse_method(&self, default: Method) -> Spanned<Method>;
    fn parse_route(&self, known_method: Option<Spanned<Method>>) -> RouteParams;
}

impl<'a, 'c> RouteDecoratorExt for MetaItemParser<'a, 'c> {
    fn bad_method(&self, sp: Span, message: &str) {
        let message = format!("{} Valid methods are: [GET, PUT, POST, DELETE, \
            OPTIONS, HEAD, TRACE, CONNECT, PATCH]", message);
        self.ctxt.span_err(sp, message.as_str());
    }

    fn parse_method(&self, default: Method) -> Spanned<Method> {
        let params = self.expect_list();
        if params.len() < 1 {
            self.bad_method(self.span, "HTTP method parameter is missing.");
            self.ctxt.span_fatal(self.span, "At least 2 arguments are required.");
        }

        // Get the method and the rest of the k = v params.
        let method_param = params.first().unwrap();

        // Check that the method parameter is a word (i.e, not a list, k/v pair).
        if !method_param.is_word() {
            self.bad_method(method_param.span,
                "Expected a valid HTTP method at this position.");
            return dummy_span(default);
        }

        // Parse the method from the string. If bad, error and return default.
        Method::from_str(method_param.name()).ok().map_or_else(|| {
            let message = format!("{} is not a valid method.", method_param.name());
            self.bad_method(method_param.span, message.as_str());
            dummy_span(default)
        }, |method| span(method, method_param.span))
    }

    // Parses the MetaItem derived from the route(...) macro.
   fn parse_route(&self, known_method: Option<Spanned<Method>>) -> RouteParams {
        let list = self.expect_list();
        let (method, kv_params) = match known_method {
            Some(method) => (method, &list[..]),
            None => (self.parse_method(Method::Get), list.split_first().unwrap().1)
        };

        // Now grab all of the required and optional parameters.
        let req: [&'static str; 1] = ["path"];
        let opt: [&'static str; 1] = ["form"];
        let kv_pairs = get_key_values(self.ctxt, self.meta_item.span,
                                      &req, &opt, kv_params);

        // Ensure we have a path, just to keep parsing and generating errors.
        let path = kv_pairs.get("path").map_or(KVSpanned::dummy("/".to_string()), |s| {
            s.clone().map(String::from)
        });

        // If there's a form parameter, ensure method is POST.
        let form = kv_pairs.get("form").map_or(None, |f| {
            if method.node != Method::Post {
                self.ctxt.span_err(f.p_span, "Use of `form` requires POST method...");
                let message = format!("...but {} was found instead.", method.node);
                self.ctxt.span_err(method.span, message.as_str());
            }

            if !(f.node.starts_with('<') && f.node.ends_with('>')) {
                self.ctxt.struct_span_err(f.p_span,
                                        "`form` cannot contain arbitrary text")
                    .help("`form` must be exactly one parameter: \"<param>\"")
                    .emit();
            }

            if f.node.chars().filter(|c| *c == '<' || *c == '>').count() != 2 {
                self.ctxt.span_err(f.p_span,
                                   "`form` must contain exactly one parameter");
            }

            Some(f.clone().map(String::from))
        });

        RouteParams {
            method: method,
            path: path,
            form: form
        }
    }

}
