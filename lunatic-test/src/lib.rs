#[allow(unused_extern_crates)]
extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_attribute]
pub fn test(_args: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::ItemFn);

    // Check if #[should_panic] attribute is present.
    let mut should_panic = None;
    let mut ignore = "";
    for attribute in input.attrs.iter() {
        if let Some(ident) = attribute.path.get_ident() {
            if ident == "ignore" {
                ignore = "#ignore_";
            }
            if ident == "should_panic" {
                // Common error message
                let error = syn::Error::new_spanned(
                    &attribute.tokens,
                    "argument must be of the form: `expected = \"error message\"`",
                )
                .to_compile_error()
                .into();

                let attribute_args = match attribute.parse_meta() {
                    Ok(args) => args,
                    Err(_) => return error,
                };

                let attribute_args = match attribute_args {
                    syn::Meta::List(attribute_args) => attribute_args,
                    syn::Meta::Path(_) => {
                        // Match any panic if no expected value is provided
                        should_panic = Some("".to_string());
                        continue;
                    }
                    _ => return error,
                };

                // should_panic can have at most one argument
                if attribute_args.nested.len() > 1 {
                    return error;
                }
                // The first argument can only be in the format 'expected = "partial matcher"'
                if let Some(argument) = attribute_args.nested.iter().next() {
                    match argument {
                        syn::NestedMeta::Meta(syn::Meta::NameValue(name_value)) => {
                            if let Some(ident) = name_value.path.get_ident() {
                                if ident != "expected" {
                                    return error;
                                }
                                match &name_value.lit {
                                    syn::Lit::Str(lit) => {
                                        // Mark function as should_panic
                                        should_panic = Some(lit.value())
                                    }
                                    _ => return error,
                                }
                            } else {
                                return error;
                            }
                        }
                        _ => return error,
                    }
                };
            }
        }
    }

    let mut export_name = format!("#lunatic_test_{}", ignore);
    if let Some(panic_str) = should_panic {
        // Escape # in panic_str
        let panic_str = panic_str.replace('#', "\\#");
        export_name = format!("{}#panic_{}#", export_name, panic_str,);
    }
    let function_name = input.sig.ident.to_string();

    quote! {
        // If not compiling for wasm32, fall back to #[test]
        #[cfg_attr(not(target_arch = "wasm32"), ::core::prelude::v1::test)]
        #[cfg_attr(
            target_arch = "wasm32",
            export_name = concat!(#export_name, module_path!(), "::", #function_name)
        )]
        #input
    }
    .into()
}
