#[allow(unused_extern_crates)]
extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;

mod abstract_process;
use abstract_process::AbstractProcessTransformer;

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

/// Add AbstractProcess behaviour to the given struct implementation with minimum
/// boilerplate code.
///
/// - Use **#\[init\]**, **#\[terminate\]**, and **#\[handle_link_trapped\]** marker macros to
/// specify methods for implementing lunatic::process::AbstractProcess.
/// - Use **#\[process_message\]** and **#\[process_request\]** marker macros to specify
/// message and request handlers.
///
/// Specifying message types is unnecessary because the macro will create wrapper
/// types for messages on all handlers. Handlers can take arbitrary number of
/// parameters and invocating them works the same as directly calling the method
/// on the struct without spawning it as a process.
///
/// # Examples
///
/// ```
/// use lunatic::process::{Message, ProcessRef, Request, StartProcess};
/// use lunatic_macros::{abstract_process, process_message, process_request};
///
/// struct Counter(u32);
///
/// #[abstract_process]
/// impl Counter {
///     fn init(_: ProcessRef<Self>, start: u32) -> Self {
///         Self(start)
///     }
///
///     #[process_message]
///     fn increment(&mut self) {
///         state.0 += 1;
///     }
///
///     #[process_request]
///     fn count(&self) -> u32 {
///         state.0
///     }
/// }
///
///
/// let counter = Counter::start(5, None);
/// counter.increment();
/// assert_eq!(counter.count(), 6);
/// ```
#[proc_macro_attribute]
pub fn abstract_process(_args: TokenStream, item: TokenStream) -> TokenStream {
    match syn::parse(item.clone()) {
        Ok(it) => AbstractProcessTransformer::new().transform(it).into(),
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
