#[allow(unused_extern_crates)]
extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;

/// Marks function to be executed by the lunatic runtime as a unit test. This is
/// a drop-in replacement for the standard `#[test]` attribute macro.
#[proc_macro_attribute]
pub fn test(_args: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::ItemFn);
    let original_input = input.clone();
    let attributes = &input.attrs;
    let span = input.span();

    // Check if #[should_panic] attribute is present.
    let mut should_panic = None;
    let mut ignore = "";
    for attribute in attributes.iter() {
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
    if let Some(ref panic_str) = should_panic {
        // Escape # in panic_str
        let panic_str = panic_str.replace('#', "\\#");
        export_name = format!("{}#panic_{}#", export_name, panic_str,);
    }
    let function_name = input.sig.ident.to_string();

    let name = input.sig.ident;
    let arguments = input.sig.inputs;
    let output = input.sig.output;
    let block = input.block;

    // `#[should_panic]` can't be combined with `Result`.
    match output {
        syn::ReturnType::Type(_, _) => {
            if should_panic.is_some() {
                return quote_spanned! {
                    span => compile_error!("functions using `#[should_panic]` must return `()`");
                }
                .into();
            }
        }
        syn::ReturnType::Default => (),
    }

    let mailbox = if !arguments.is_empty() {
        quote! { lunatic::Mailbox::new() }
    } else {
        quote! {}
    };

    let wasm32_test = quote! {
        fn #name() {
            fn __with_mailbox(#arguments) #output {
                #block
            }
            let result = unsafe { __with_mailbox(#mailbox) };
            lunatic::test::assert_test_result(result);
        }
    };

    quote! {
        // If not compiling for wasm32, fall back to #[test]
        #[cfg_attr(not(target_arch = "wasm32"), ::core::prelude::v1::test)]
        #[cfg(not(target_arch = "wasm32"))]
        #original_input

        #[cfg_attr(
            target_arch = "wasm32",
            export_name = concat!(#export_name, module_path!(), "::", #function_name)
        )]
        #[cfg(target_arch = "wasm32")]
        #wasm32_test
    }
    .into()
}
