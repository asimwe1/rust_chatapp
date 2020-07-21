use devise::{syn, Spanned, Result, FromMeta, Diagnostic};
use devise::ext::SpanDiagnosticExt;

use crate::proc_macro2::{TokenStream, Span};
use crate::http_codegen::Status;
use crate::syn_ext::{IdentExt, ReturnTypeExt, TokenStreamExt};
use self::syn::{Attribute, parse::Parser};
use crate::{CATCH_FN_PREFIX, CATCH_STRUCT_PREFIX};

/// The raw, parsed `#[catch(code)]` attribute.
#[derive(Debug, FromMeta)]
struct CatchAttribute {
    #[meta(naked)]
    status: Status
}

/// This structure represents the parsed `catch` attribute an associated items.
struct CatchParams {
    /// The status associated with the code in the `#[catch(code)]` attribute.
    status: Status,
    /// The function that was decorated with the `catch` attribute.
    function: syn::ItemFn,
}

fn parse_params(
    args: TokenStream,
    input: proc_macro::TokenStream
) -> Result<CatchParams> {
    let function: syn::ItemFn = syn::parse(input)
        .map_err(Diagnostic::from)
        .map_err(|diag| diag.help("`#[catch]` can only be used on functions"))?;

    let full_attr = quote!(#[catch(#args)]);
    let attrs = Attribute::parse_outer.parse2(full_attr)?;
    let attribute = match CatchAttribute::from_attrs("catch", &attrs) {
        Some(result) => result.map_err(|d| {
            d.help("`#[catch]` expects a single status integer, e.g.: #[catch(404)]")
        })?,
        None => return Err(Span::call_site().error("internal error: bad attribute"))
    };

    Ok(CatchParams { status: attribute.status, function })
}

pub fn _catch(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream
) -> Result<TokenStream> {
    // Parse and validate all of the user's input.
    let catch = parse_params(args.into(), input)?;

    // Gather everything we'll need to generate the catcher.
    let user_catcher_fn = &catch.function;
    let user_catcher_fn_name = catch.function.sig.ident.clone();
    let generated_struct_name = user_catcher_fn_name.prepend(CATCH_STRUCT_PREFIX);
    let generated_fn_name = user_catcher_fn_name.prepend(CATCH_FN_PREFIX);
    let (vis, status) = (&catch.function.vis, &catch.status);
    let status_code = status.0.code;

    // Variables names we'll use and reuse.
    define_vars_and_mods!(catch.function.span().into() =>
        req, _Box, Request, Response, CatcherFuture);

    // Determine the number of parameters that will be passed in.
    if catch.function.sig.inputs.len() > 1 {
        return Err(catch.function.sig.paren_token.span
            .error("invalid number of arguments: must be zero or one")
            .help("catchers may optionally take an argument of type `&Request`"));
    }

    // TODO: It would be nice if this worked! Alas, either there is a rustc bug
    // that prevents this from working (error on `Output` type of `Future`), or
    // this simply isn't possible with `async fn`.
    // // Typecheck the catcher function if it has arguments.
    // user_catcher_fn_name.set_span(catch.function.sig.paren_token.span.into());
    // let user_catcher_fn_call = catch.function.sig.inputs.first()
    //     .map(|arg| {
    //         let ty = quote!(fn(&#Request) -> _).respanned(Span::call_site().into());
    //         let req = req.respanned(arg.span().into());
    //         quote!({
    //             let #user_catcher_fn_name: #ty = #user_catcher_fn_name;
    //             #user_catcher_fn_name(#req)
    //         })
    //     })
    //     .unwrap_or_else(|| quote!(#user_catcher_fn_name()));
    //
    // let catcher_response = quote_spanned!(return_type_span => {
    //     let ___responder = #user_catcher_fn_call #dot_await;
    //     ::rocket::response::Responder::respond_to(___responder, #req)?
    // });

    // This ensures that "Responder not implemented" points to the return type.
    let return_type_span = catch.function.sig.output.ty()
        .map(|ty| ty.span().into())
        .unwrap_or(Span::call_site().into());

    // Set the `req` span to that of the arg for a correct `Wrong type` span.
    let input = catch.function.sig.inputs.first()
        .map(|arg| match arg {
            syn::FnArg::Receiver(_) => req.respanned(arg.span()),
            syn::FnArg::Typed(a) => req.respanned(a.ty.span())
        });

    // We append `.await` to the function call if this is `async`.
    let dot_await = catch.function.sig.asyncness
        .map(|a| quote_spanned!(a.span().into() => .await));

    let catcher_response = quote_spanned!(return_type_span => {
        let ___responder = #user_catcher_fn_name(#input) #dot_await;
        ::rocket::response::Responder::respond_to(___responder, #req)?
    });

    // Generate the catcher, keeping the user's input around.
    Ok(quote! {
        #user_catcher_fn

        /// Rocket code generated wrapping catch function.
        #[doc(hidden)]
        #vis fn #generated_fn_name<'_b>(#req: &'_b #Request) -> #CatcherFuture<'_b> {
            #_Box::pin(async move {
                let __response = #catcher_response;
                #Response::build()
                    .status(#status)
                    .merge(__response)
                    .ok()
            })
        }

        /// Rocket code generated static catcher info.
        #[doc(hidden)]
        #[allow(non_upper_case_globals)]
        #vis static #generated_struct_name: ::rocket::StaticCatchInfo =
            ::rocket::StaticCatchInfo {
                code: #status_code,
                handler: #generated_fn_name,
            };
    })
}

pub fn catch_attribute(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream
) -> TokenStream {
    _catch(args, input).unwrap_or_else(|d| d.emit_as_item_tokens())
}
