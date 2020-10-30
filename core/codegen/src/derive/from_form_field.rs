use devise::{*, ext::SpanDiagnosticExt};

use crate::exports::*;
use crate::proc_macro2::TokenStream;
use crate::syn_ext::NameSource;

#[derive(FromMeta)]
pub struct FieldAttr {
    value: NameSource,
}

pub fn derive_from_form_field(input: proc_macro::TokenStream) -> TokenStream {
    DeriveGenerator::build_for(input, quote!(impl<'__v> #_form::FromFormField<'__v>))
        .support(Support::Enum)
        .validator(ValidatorBuild::new()
            // We only accepts C-like enums with at least one variant.
            .fields_validate(|_, fields| {
                if !fields.is_empty() {
                    return Err(fields.span().error("variants cannot have fields"));
                }

                Ok(())
            })
            .enum_validate(|_, data| {
                if data.variants.is_empty() {
                    return Err(data.span().error("enum must have at least one variant"));
                }

                Ok(())
            })
        )
        // TODO: Devise should have a try_variant_map.
        .inner_mapper(MapperBuild::new()
            .try_enum_map(|_, data| {
                let variant_name_sources = data.variants()
                    .map(|v| FieldAttr::one_from_attrs("field", &v.attrs).map(|o| {
                        o.map(|f| f.value).unwrap_or_else(|| v.ident.clone().into())
                    }))
                    .collect::<Result<Vec<NameSource>>>()?;

                let variant_name = variant_name_sources.iter()
                    .map(|n| n.name())
                    .collect::<Vec<_>>();

                let builder = data.variants()
                    .map(|v| v.builder(|_| unreachable!("fieldless")));

                let (_ok, _cow) = (std::iter::repeat(_Ok), std::iter::repeat(_Cow));
                Ok(quote! {
                    fn from_value(
                        __f: #_form::ValueField<'__v>
                    ) -> Result<Self, #_form::Errors<'__v>> {
                        #[allow(unused_imports)]
                        use #_http::uncased::AsUncased;

                        #(
                            if __f.value.as_uncased() == #variant_name {
                                return #_ok(#builder);
                            }
                        )*

                        const OPTS: &'static [#_Cow<'static, str>] =
                            &[#(#_cow::Borrowed(#variant_name)),*];

                        let _error = #_form::Error::from(OPTS)
                            .with_name(__f.name)
                            .with_value(__f.value);

                        #_Err(_error)?
                    }
                })
            })
        )
        .to_tokens()
}
