#[allow(unused_extern_crates)]
extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_attribute]
pub fn main(_args: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::ItemFn);

    if input.sig.ident != "main" || input.sig.inputs.len() != 1 {
        let msg = "must be on a `main` function with 1 argument of type Mailbox<T>";
        return syn::Error::new_spanned(&input.sig.ident, msg)
            .to_compile_error()
            .into();
    }

    parse(input, false).unwrap_or_else(|e| e.to_compile_error().into())
}

#[proc_macro_attribute]
pub fn test(_args: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::ItemFn);

    if input.sig.inputs.len() != 1 {
        let msg = "test functions accept only 1 argument of type Mailbox<T>";
        return syn::Error::new_spanned(&input.sig.ident, msg)
            .to_compile_error()
            .into();
    }

    for attr in &input.attrs {
        if attr.path.is_ident("test") {
            let msg = "second test attribute is supplied";
            return syn::Error::new_spanned(&attr, msg)
                .to_compile_error()
                .into();
        }
    }

    parse(input, true).unwrap_or_else(|e| e.to_compile_error().into())
}

#[allow(dead_code)] // Rust-analyzer fix
fn parse(input: syn::ItemFn, is_test: bool) -> Result<TokenStream, syn::Error> {
    let header = if is_test {
        quote! {
            #[::core::prelude::v1::test]
        }
    } else {
        quote! {}
    };

    let name = input.sig.ident;
    let arguments = input.sig.inputs;
    let block = input.block;
    let body = quote! {
        fn #name() {
            fn ______hidden(#arguments) {
                #block
            }
            unsafe { ______hidden(lunatic::Mailbox::new()) }
            ;
        }
    };

    let result = quote! {
        #header
        #body
    };

    Ok(result.into())
}
