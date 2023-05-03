use darling::FromAttributes;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{spanned::Spanned, DeriveInput};

#[derive(Default, FromAttributes)]
#[darling(attributes(lunatic))]
struct Attrs {
    process_name: Option<String>,
}

pub struct ProcessNameDerive {
    attrs: Result<Attrs, darling::Error>,
    ident: syn::Ident,
}

impl syn::parse::Parse for ProcessNameDerive {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let input: DeriveInput = input.parse()?;
        let attrs = Attrs::from_attributes(&input.attrs);

        let has_generics = input.generics.type_params().next().is_some();
        let has_override = attrs
            .as_ref()
            .map(|attrs| attrs.process_name.is_some())
            .unwrap_or(false);
        if has_generics && !has_override {
            return Err(syn::Error::new(
                input.generics.span(),
                "ProcessName derive does not support generics.\nEither implement ProcessName manually, or use the #[lunatic(process_name = \"...\")] attribute",
            ));
        }

        Ok(ProcessNameDerive {
            attrs,
            ident: input.ident,
        })
    }
}

impl ToTokens for ProcessNameDerive {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { attrs, ident } = self;

        let attrs = match attrs {
            Ok(attrs) => attrs,
            Err(err) => {
                tokens.append_all(err.clone().write_errors());
                return;
            }
        };

        let process_name_impl = match &attrs.process_name {
            Some(process_name) => quote! { #process_name },
            None => {
                let ident_str = ident.to_string();
                quote! {
                    std::concat!(
                        std::env!("CARGO_PKG_NAME"),
                        "@",
                        std::env!("CARGO_PKG_VERSION"),
                        "::",
                        std::module_path!(),
                        "::",
                        #ident_str
                    )
                }
            }
        };

        tokens.append_all(quote! {
            impl lunatic::ProcessName for #ident {
                fn process_name(&self) -> &str {
                    #process_name_impl
                }
            }
        });
    }
}
