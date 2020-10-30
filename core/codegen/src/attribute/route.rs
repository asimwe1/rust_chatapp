use std::sync::atomic::{AtomicUsize, Ordering};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use devise::{syn, Spanned, SpanWrapped, Result, FromMeta, Diagnostic};
use devise::ext::{SpanDiagnosticExt, TypeExt};
use indexmap::IndexSet;

use crate::proc_macro_ext::{Diagnostics, StringLit};
use crate::syn_ext::{IdentExt, NameSource};
use crate::proc_macro2::{TokenStream, Span};
use crate::http_codegen::{Method, MediaType, RoutePath, DataSegment, Optional};
use crate::attribute::segments::{Source, Kind, Segment};

use crate::{URI_MACRO_PREFIX, ROCKET_PARAM_PREFIX};

/// The raw, parsed `#[route]` attribute.
#[derive(Debug, FromMeta)]
struct RouteAttribute {
    #[meta(naked)]
    method: SpanWrapped<Method>,
    path: RoutePath,
    data: Option<SpanWrapped<DataSegment>>,
    format: Option<MediaType>,
    rank: Option<isize>,
}

/// The raw, parsed `#[method]` (e.g, `get`, `put`, `post`, etc.) attribute.
#[derive(Debug, FromMeta)]
struct MethodRouteAttribute {
    #[meta(naked)]
    path: RoutePath,
    data: Option<SpanWrapped<DataSegment>>,
    format: Option<MediaType>,
    rank: Option<isize>,
}

/// This structure represents the parsed `route` attribute and associated items.
#[derive(Debug)]
struct Route {
    /// The attribute: `#[get(path, ...)]`.
    attribute: RouteAttribute,
    /// The function the attribute decorated, i.e, the handler.
    function: syn::ItemFn,
    /// The non-static parameters declared in the route segments.
    segments: IndexSet<Segment>,
    /// The parsed inputs to the user's function. The name is the param as the
    /// user wrote it, while the ident is the identifier that should be used
    /// during code generation, the `rocket_ident`.
    inputs: Vec<(NameSource, syn::Ident, syn::Type)>,
}

impl Route {
    fn find_input<T>(&self, name: &T) -> Option<&(NameSource, syn::Ident, syn::Type)>
        where T: PartialEq<NameSource>
    {
        self.inputs.iter().find(|(n, ..)| name == n)
    }
}

fn parse_route(attr: RouteAttribute, function: syn::ItemFn) -> Result<Route> {
    // Gather diagnostics as we proceed.
    let mut diags = Diagnostics::new();

    // Emit a warning if a `data` param was supplied for non-payload methods.
    if let Some(ref data) = attr.data {
        if !attr.method.0.supports_payload() {
            let msg = format!("'{}' does not typically support payloads", attr.method.0);
            // FIXME(diag: warning)
            data.full_span.warning("`data` used with non-payload-supporting method")
                .span_note(attr.method.span, msg)
                .emit_as_item_tokens();
        }
    }

    // Collect non-wild dynamic segments in an `IndexSet`, checking for dups.
    let mut segments: IndexSet<Segment> = IndexSet::new();
    fn dup_check<'a, I>(set: &mut IndexSet<Segment>, iter: I, diags: &mut Diagnostics)
        where I: Iterator<Item = &'a Segment>
    {
        for segment in iter.filter(|s| s.is_dynamic()) {
            let span = segment.span;
            if let Some(previous) = set.replace(segment.clone()) {
                diags.push(span.error(format!("duplicate parameter: `{}`", previous.name))
                    .span_note(previous.span, "previous parameter with the same name here"))
            }
        }
    }

    dup_check(&mut segments, attr.path.path.iter().filter(|s| !s.is_wild()), &mut diags);
    attr.path.query.as_ref().map(|q| dup_check(&mut segments, q.iter(), &mut diags));
    dup_check(&mut segments, attr.data.as_ref().map(|s| &s.value.0).into_iter(), &mut diags);

    // Check the validity of function arguments.
    let mut inputs = vec![];
    let mut fn_segments: IndexSet<Segment> = IndexSet::new();
    for input in &function.sig.inputs {
        let help = "all handler arguments must be of the form: `ident: Type`";
        let span = input.span();
        let (ident, ty) = match input {
            syn::FnArg::Typed(arg) => match *arg.pat {
                syn::Pat::Ident(ref pat) => (&pat.ident, &arg.ty),
                syn::Pat::Wild(_) => {
                    diags.push(span.error("handler arguments cannot be ignored").help(help));
                    continue;
                }
                _ => {
                    diags.push(span.error("invalid use of pattern").help(help));
                    continue;
                }
            }
            // Other cases shouldn't happen since we parsed an `ItemFn`.
            _ => {
                diags.push(span.error("invalid handler argument").help(help));
                continue;
            }
        };

        let rocket_ident = ident.prepend(ROCKET_PARAM_PREFIX);
        inputs.push((ident.clone().into(), rocket_ident, ty.with_stripped_lifetimes()));
        fn_segments.insert(ident.into());
    }

    // Check that all of the declared parameters are function inputs.
    let span = function.sig.paren_token.span;
    for missing in segments.difference(&fn_segments) {
        diags.push(missing.span.error("unused dynamic parameter")
            .span_note(span, format!("expected argument named `{}` here", missing.name)))
    }

    diags.head_err_or(Route { attribute: attr, function, inputs, segments })
}

fn param_expr(seg: &Segment, ident: &syn::Ident, ty: &syn::Type) -> TokenStream {
    let i = seg.index.expect("dynamic parameters must be indexed");
    let span = ident.span().join(ty.span()).unwrap_or_else(|| ty.span());
    let name = ident.to_string();

    define_spanned_export!(span =>
        __req, __data, _log, _request, _None, _Some, _Ok, _Err, Outcome
    );

    // All dynamic parameter should be found if this function is being called;
    // that's the point of statically checking the URI parameters.
    let internal_error = quote!({
        #_log::error("Internal invariant error: expected dynamic parameter not found.");
        #_log::error("Please report this error to the Rocket issue tracker.");
        #Outcome::Forward(#__data)
    });

    // Returned when a dynamic parameter fails to parse.
    let parse_error = quote!({
        #_log::warn_(&format!("Failed to parse '{}': {:?}", #name, __error));
        #Outcome::Forward(#__data)
    });

    let expr = match seg.kind {
        Kind::Single => quote_spanned! { span =>
            match #__req.routed_segment(#i) {
                #_Some(__s) => match <#ty as #_request::FromParam>::from_param(__s) {
                    #_Ok(__v) => __v,
                    #_Err(__error) => return #parse_error,
                },
                #_None => return #internal_error
            }
        },
        Kind::Multi => quote_spanned! { span =>
            match <#ty as #_request::FromSegments>::from_segments(#__req.routed_segments(#i..)) {
                #_Ok(__v) => __v,
                #_Err(__error) => return #parse_error,
            }
        },
        Kind::Static => return quote!()
    };

    quote! {
        let #ident: #ty = #expr;
    }
}

fn data_expr(ident: &syn::Ident, ty: &syn::Type) -> TokenStream {
    let span = ident.span().join(ty.span()).unwrap_or_else(|| ty.span());
    define_spanned_export!(span => __req, __data, FromData, Outcome);

    quote_spanned! { span =>
        let __outcome = <#ty as #FromData>::from_data(#__req, #__data).await;

        let #ident: #ty = match __outcome {
            #Outcome::Success(__d) => __d,
            #Outcome::Forward(__d) => return #Outcome::Forward(__d),
            #Outcome::Failure((__c, _)) => return #Outcome::Failure(__c),
        };
    }
}

fn query_exprs(route: &Route) -> Option<TokenStream> {
    use devise::ext::{Split2, Split6};

    define_spanned_export!(Span::call_site() =>
        __req, __data, _log, _form, Outcome, _Ok, _Err, _Some, _None
    );

    let query_segments = route.attribute.path.query.as_ref()?;

    // Record all of the static parameters for later filtering.
    let (raw_name, raw_value) = query_segments.iter()
        .filter(|s| !s.is_dynamic())
        .map(|s| {
            let name = s.name.name();
            match name.find('=') {
                Some(i) => (&name[..i], &name[i + 1..]),
                None => (name, "")
            }
        })
        .split2();

    // Now record all of the dynamic parameters.
    let (name, matcher, ident, init_expr, push_expr, finalize_expr) = query_segments.iter()
        .filter(|s| s.is_dynamic())
        .map(|s| (s, s.name.name(), route.find_input(&s.name).expect("dynamic has input")))
        .map(|(seg, name, (_, ident, ty))| {
            let matcher = match seg.kind {
                Kind::Multi => quote_spanned!(seg.span => _),
                _ => quote_spanned!(seg.span => #name)
            };

            let span = ty.span();
            define_spanned_export!(span => FromForm, _form);

            let ty = quote_spanned!(span => <#ty as #FromForm>);
            let i = ident.clone().with_span(span);
            let init = quote_spanned!(span => #ty::init(#_form::Options::Lenient));
            let finalize = quote_spanned!(span => #ty::finalize(#i));
            let push = match seg.kind {
                Kind::Multi => quote_spanned!(span => #ty::push_value(&mut #i, _f)),
                _ => quote_spanned!(span => #ty::push_value(&mut #i, _f.shift())),
            };

            (name, matcher, ident, init, push, finalize)
        })
        .split6();

    #[allow(non_snake_case)]
    Some(quote! {
        let mut _e = #_form::Errors::new();
        #(let mut #ident = #init_expr;)*

        for _f in #__req.query_fields() {
            let _raw = (_f.name.source().as_str(), _f.value);
            let _key = _f.name.key_lossy().as_str();
            match (_raw, _key) {
                // Skip static parameters so <param..> doesn't see them.
                #(((#raw_name, #raw_value), _) => { /* skip */ },)*
                #((_, #matcher) => #push_expr,)*
                _ => { /* in case we have no trailing, ignore all else */ },
            }
        }

        #(
            let #ident = match #finalize_expr {
                #_Ok(_v) => #_Some(_v),
                #_Err(_err) => {
                    _e.extend(_err.with_name(#_form::NameView::new(#name)));
                    #_None
                },
            };
        )*

        if !_e.is_empty() {
            #_log::warn_("query string failed to match declared route");
            for _err in _e { #_log::warn_(_err); }
            return #Outcome::Forward(#__data);
        }

        #(let #ident = #ident.unwrap();)*
    })
}

fn request_guard_expr(ident: &syn::Ident, ty: &syn::Type) -> TokenStream {
    let span = ident.span().join(ty.span()).unwrap_or_else(|| ty.span());
    define_spanned_export!(span => __req, __data, _request, Outcome);
    quote_spanned! { span =>
        let #ident: #ty = match <#ty as #_request::FromRequest>::from_request(#__req).await {
            #Outcome::Success(__v) => __v,
            #Outcome::Forward(_) => return #Outcome::Forward(#__data),
            #Outcome::Failure((__c, _)) => return #Outcome::Failure(__c),
        };
    }
}

fn generate_internal_uri_macro(route: &Route) -> TokenStream {
    // Keep a global counter (+ thread ID later) to generate unique ids.
    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    let dynamic_args = route.segments.iter()
        .filter(|seg| seg.source == Source::Path || seg.source == Source::Query)
        .filter(|seg| seg.kind != Kind::Static)
        .map(|seg| &seg.name)
        .map(|seg_name| route.find_input(seg_name).unwrap())
        .map(|(name, _, ty)| (name.ident(), ty))
        .map(|(ident, ty)| quote!(#ident: #ty));

    let mut hasher = DefaultHasher::new();
    route.function.sig.ident.hash(&mut hasher);
    route.attribute.path.origin.0.path().hash(&mut hasher);
    std::process::id().hash(&mut hasher);
    std::thread::current().id().hash(&mut hasher);
    COUNTER.fetch_add(1, Ordering::AcqRel).hash(&mut hasher);

    let generated_macro_name = route.function.sig.ident.prepend(URI_MACRO_PREFIX);
    let inner_generated_macro_name = generated_macro_name.append(&hasher.finish().to_string());
    let route_uri = route.attribute.path.origin.0.to_string();

    quote_spanned! { Span::call_site() =>
        #[doc(hidden)]
        #[macro_export]
        macro_rules! #inner_generated_macro_name {
            ($($token:tt)*) => {{
                extern crate std;
                extern crate rocket;
                rocket::rocket_internal_uri!(#route_uri, (#(#dynamic_args),*), $($token)*)
            }};
        }

        #[doc(hidden)]
        pub use #inner_generated_macro_name as #generated_macro_name;
    }
}

fn generate_respond_expr(route: &Route) -> TokenStream {
    let ret_span = match route.function.sig.output {
        syn::ReturnType::Default => route.function.sig.ident.span(),
        syn::ReturnType::Type(_, ref ty) => ty.span().into()
    };

    define_spanned_export!(ret_span => __req, _handler);
    let user_handler_fn_name = &route.function.sig.ident;
    let parameter_names = route.inputs.iter()
        .map(|(_, rocket_ident, _)| rocket_ident);

    let _await = route.function.sig.asyncness
        .map(|a| quote_spanned!(a.span().into() => .await));

    let responder_stmt = quote_spanned! { ret_span =>
        let ___responder = #user_handler_fn_name(#(#parameter_names),*) #_await;
    };

    quote_spanned! { ret_span =>
        #responder_stmt
        #_handler::Outcome::from(#__req, ___responder)
    }
}

fn codegen_route(route: Route) -> Result<TokenStream> {
    // Generate the declarations for path, data, and request guard parameters.
    let mut data_stmt = None;
    let mut req_guard_definitions = vec![];
    let mut parameter_definitions = vec![];
    for (name, rocket_ident, ty) in &route.inputs {
        let fn_segment: Segment = name.ident().into();
        match route.segments.get(&fn_segment) {
            Some(seg) if seg.source == Source::Path => {
                parameter_definitions.push(param_expr(seg, rocket_ident, &ty));
            }
            Some(seg) if seg.source == Source::Data => {
                // the data statement needs to come last, so record it specially
                data_stmt = Some(data_expr(rocket_ident, &ty));
            }
            Some(_) => continue, // handle query parameters later
            None => {
                req_guard_definitions.push(request_guard_expr(rocket_ident, &ty));
            }
        };
    }

    // Generate the declarations for query parameters.
    if let Some(exprs) = query_exprs(&route) {
        parameter_definitions.push(exprs);
    }

    // Gather everything we need.
    use crate::exports::{
        __req, __data, _Box, Request, Data, Route, StaticRouteInfo, HandlerFuture
    };

    let (vis, user_handler_fn) = (&route.function.vis, &route.function);
    let user_handler_fn_name = &user_handler_fn.sig.ident;
    let generated_internal_uri_macro = generate_internal_uri_macro(&route);
    let generated_respond_expr = generate_respond_expr(&route);

    let method = route.attribute.method;
    let path = route.attribute.path.origin.0.to_string();
    let rank = Optional(route.attribute.rank);
    let format = Optional(route.attribute.format);

    Ok(quote! {
        #user_handler_fn

        #[doc(hidden)]
        #[allow(non_camel_case_types)]
        /// Rocket code generated proxy structure.
        #vis struct #user_handler_fn_name {  }

        /// Rocket code generated proxy static conversion implementation.
        impl From<#user_handler_fn_name> for #StaticRouteInfo {
            #[allow(non_snake_case, unreachable_patterns, unreachable_code)]
            fn from(_: #user_handler_fn_name) -> #StaticRouteInfo {
                fn monomorphized_function<'_b>(
                    #__req: &'_b #Request<'_>,
                    #__data: #Data
                ) -> #HandlerFuture<'_b> {
                    #_Box::pin(async move {
                        #(#req_guard_definitions)*
                        #(#parameter_definitions)*
                        #data_stmt

                        #generated_respond_expr
                    })
                }

                #StaticRouteInfo {
                    name: stringify!(#user_handler_fn_name),
                    method: #method,
                    path: #path,
                    handler: monomorphized_function,
                    format: #format,
                    rank: #rank,
                }
            }
        }

        /// Rocket code generated proxy conversion implementation.
        impl From<#user_handler_fn_name> for #Route {
            #[inline]
            fn from(_: #user_handler_fn_name) -> #Route {
                #StaticRouteInfo::from(#user_handler_fn_name {}).into()
            }
        }

        /// Rocket code generated wrapping URI macro.
        #generated_internal_uri_macro
    }.into())
}

fn complete_route(args: TokenStream, input: TokenStream) -> Result<TokenStream> {
    let function: syn::ItemFn = syn::parse2(input)
        .map_err(|e| Diagnostic::from(e))
        .map_err(|diag| diag.help("`#[route]` can only be used on functions"))?;

    let attr_tokens = quote!(route(#args));
    let attribute = RouteAttribute::from_meta(&syn::parse2(attr_tokens)?)?;
    codegen_route(parse_route(attribute, function)?)
}

fn incomplete_route(
    method: crate::http::Method,
    args: TokenStream,
    input: TokenStream
) -> Result<TokenStream> {
    let method_str = method.to_string().to_lowercase();
    // FIXME(proc_macro): there should be a way to get this `Span`.
    let method_span = StringLit::new(format!("#[{}]", method), Span::call_site())
        .subspan(2..2 + method_str.len());

    let method_ident = syn::Ident::new(&method_str, method_span.into());

    let function: syn::ItemFn = syn::parse2(input)
        .map_err(|e| Diagnostic::from(e))
        .map_err(|d| d.help(format!("#[{}] can only be used on functions", method_str)))?;

    let full_attr = quote!(#method_ident(#args));
    let method_attribute = MethodRouteAttribute::from_meta(&syn::parse2(full_attr)?)?;

    let attribute = RouteAttribute {
        method: SpanWrapped {
            full_span: method_span, span: method_span, value: Method(method)
        },
        path: method_attribute.path,
        data: method_attribute.data,
        format: method_attribute.format,
        rank: method_attribute.rank,
    };

    codegen_route(parse_route(attribute, function)?)
}

pub fn route_attribute<M: Into<Option<crate::http::Method>>>(
    method: M,
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream
) -> TokenStream {
    let result = match method.into() {
        Some(method) => incomplete_route(method, args.into(), input.into()),
        None => complete_route(args.into(), input.into())
    };

    result.unwrap_or_else(|diag| diag.emit_as_item_tokens())
}
