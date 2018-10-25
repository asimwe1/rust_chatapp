use proc_macro::{Span, TokenStream};
use derive_utils::*;

use derive::from_form::Form;

const NO_EMPTY_FIELDS: &str = "fieldless structs or variants are not supported";
const NO_NULLARY: &str = "nullary items are not supported";
const NO_EMPTY_ENUMS: &str = "empty enums are not supported";
const ONLY_ONE_UNNAMED: &str = "tuple structs or variants must have exactly one field";

fn validate_fields(fields: Fields, parent_span: Span) -> Result<()> {
    if fields.count() == 0 {
        return Err(parent_span.error(NO_EMPTY_FIELDS))
    } else if fields.are_unnamed() && fields.count() > 1 {
        return Err(fields.span().error(ONLY_ONE_UNNAMED));
    } else if fields.are_unit() {
        return Err(parent_span.error(NO_NULLARY));
    }

    Ok(())
}

fn validate_struct(gen: &DeriveGenerator, data: Struct) -> Result<()> {
    validate_fields(data.fields(), gen.input.span())
}

fn validate_enum(gen: &DeriveGenerator, data: Enum) -> Result<()> {
    if data.variants().count() == 0 {
        return Err(gen.input.span().error(NO_EMPTY_ENUMS));
    }

    for variant in data.variants() {
        validate_fields(variant.fields(), variant.span())?;
    }

    Ok(())
}

pub fn derive_uri_display(input: TokenStream) -> TokenStream {
    DeriveGenerator::build_for(input, "::rocket::http::uri::UriDisplay")
        .generic_support(GenericSupport::Type | GenericSupport::Lifetime)
        .data_support(DataSupport::Struct | DataSupport::Enum)
        .validate_enum(validate_enum)
        .validate_struct(validate_struct)
        .map_type_generic(|_, ident, _| quote!(#ident : ::rocket::http::uri::UriDisplay))
        .function(|_, inner| quote! {
            fn fmt(&self, f: &mut ::rocket::http::uri::Formatter) -> ::std::fmt::Result {
                #inner
                Ok(())
            }
        })
        .try_map_field(|_, field| {
            let span = field.span().into();
            let accessor = field.accessor();
            let tokens = if let Some(ref ident) = field.ident {
                let name = Form::from_attrs("form", &field.attrs)
                    .map(|result| result.map(|form| form.field.name))
                    .unwrap_or_else(|| Ok(ident.to_string()))?;

                quote_spanned!(span => f.write_named_value(#name, &#accessor)?;)
            } else {
                quote_spanned!(span => f.write_value(&#accessor)?;)
            };

            Ok(tokens)
        })
        .to_tokens()
}
