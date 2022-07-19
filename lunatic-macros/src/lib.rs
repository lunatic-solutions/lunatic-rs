#![feature(box_patterns)]
#[allow(unused_extern_crates)]
extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;

mod abstract_process;
use abstract_process::render_abstract_process;

#[proc_macro_attribute]
pub fn main(_args: TokenStream, item: TokenStream) -> TokenStream {
    let input: syn::ItemFn = match syn::parse(item.clone()) {
        Ok(it) => it,
        Err(e) => return token_stream_with_error(item, e),
    };

    if input.sig.ident != "main" || input.sig.inputs.len() != 1 {
        let msg = "must be on a `main` function with 1 argument of type Mailbox<T>";
        return syn::Error::new_spanned(&input.sig.ident, msg)
            .to_compile_error()
            .into();
    }

    let name = input.sig.ident;
    let arguments = input.sig.inputs;
    let block = input.block;

    quote! {
        fn #name() {
            fn __with_mailbox(#arguments) {
                #block
            }
            unsafe { __with_mailbox(lunatic::Mailbox::new()) };
        }
    }
    .into()
}

#[proc_macro_attribute]
pub fn abstract_process(_args: TokenStream, item: TokenStream) -> TokenStream {
    match syn::parse(item.clone()) {
        Ok(it) => render_abstract_process(it).into(),
        Err(e) => token_stream_with_error(item, e),
    }
}

#[proc_macro_attribute]
pub fn init(_args: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn terminate(_args: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn handle_link_trapped(_args: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn process_message(_args: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn process_request(_args: TokenStream, item: TokenStream) -> TokenStream {
    item
}

fn token_stream_with_error(mut tokens: TokenStream, error: syn::Error) -> TokenStream {
    tokens.extend(TokenStream::from(error.into_compile_error()));
    tokens
}
