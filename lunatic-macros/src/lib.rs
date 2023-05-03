#[allow(unused_extern_crates)]
extern crate proc_macro;
use proc_macro::TokenStream;
use process_name::ProcessNameDerive;
use quote::{quote, ToTokens};
use syn::parse_macro_input;

mod abstract_process;
mod process_name;

/// Marks the main function to be executed by the lunatic runtime as the root
/// process.
///
/// Note: The macro can only be used on `main` function with 1 argument of type
/// `Mailbox<T>`.
///
/// # Example
/// ```ignore
/// #[lunatic::main]
/// fn main(_: Mailbox<()>) {
///     println!("Hello, world!");
/// }
/// ```
#[allow(clippy::needless_doctest_main)]
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

    let arguments = input.sig.inputs;
    let block = input.block;

    quote! {
        fn main() {
            fn __with_mailbox(#arguments) {
                #block
            }
            unsafe { __with_mailbox(lunatic::Mailbox::new()) };
        }
    }
    .into()
}

/// Add [`AbstractProcess`] behavior to the given struct implementation with
/// minimum boilerplate code.
///
/// - Use `#[init]`, `#[terminate]`, and `#[handle_link_trapped]` attributes to
/// specify methods for implementing [`AbstractProcess`].
/// - Use `#[handle_message]`, `#[handle_request]` and
///   `#[handle_deferred_request]` attributes to specify message and request
///   handlers.
///
/// Specifying message types is unnecessary because the macro will create
/// wrapper types for messages on all handlers. Handlers can take an arbitrary
/// number of parameters and invoking them works the same as directly calling
/// the method on the struct without spawning it as a process.
///
/// A trait is generated and defaults to private and follows the name of your
/// type with `Handler` added as a suffix. To rename or change the visibility of
/// the generated trait, you can use the `trait_name` and `visbility` arguments
/// with `#[abstract_process(trait_name = "MyHandler", visibility = pub)]`.
///
/// # Examples
///
/// ```ignore
/// use lunatic::ap::{AbstractProcess, Config, DeferredResponse};
/// use lunatic::{abstract_process, Mailbox, Tag};
///
/// struct Counter(u32);
///
/// #[abstract_process]
/// impl Counter {
///     #[init]
///     fn init(_: Config<Self>, start: u32) -> Result<Self, ()> {
///         Ok(Self(start))
///     }
///
///     #[terminate]
///     fn terminate(self) {
///         println!("Shutdown process");
///     }
///
///     #[handle_link_death]
///     fn handle_link_death(&self, _tag: Tag) {
///         println!("Link trapped");
///     }
///
///     #[handle_message]
///     fn increment(&mut self) {
///         self.0 += 1;
///     }
///
///     #[handle_request]
///     fn count(&self) -> u32 {
///         self.0
///     }
///
///     #[handle_deferred_request]
///     fn add_to_count(&self, a: u32, b: u32, dr: DeferredResponse<u32, Self>) {
///         dr.send_response(self.0 + a + b)
///     }
/// }
///
/// let counter = Counter::link().start(0).unwrap();
/// counter.increment();
/// assert_eq!(counter.count(), 1);
/// counter.increment();
/// assert_eq!(
///         counter
///             .with_timeout(Duration::from_millis(10))
///             .add_to_count(1, 1)
///             .unwrap(),
///         4
///     );
/// ```
/// [`AbstractProcess`]: process/trait.AbstractProcess.html
#[proc_macro_attribute]
pub fn abstract_process(args: TokenStream, item: TokenStream) -> TokenStream {
    match abstract_process::AbstractProcess::new(args, item) {
        Ok(abstract_process) => abstract_process.expand().into(),
        Err(err) => err.into_compile_error().into(),
    }
}

/// ProcessName implements the `lunatic::ProcessName` trait by generating a unique name
/// in the following format:
///
/// ```text
/// "<pkg_name>@<pkg_version>::<module_path>::<ident>"
/// ```
///
/// # Example
///
/// ```ignore
/// #[derive(ProcessName)]
/// struct LoggingProcess;
///
/// assert_eq!(LoggingProcess.process_name(), "lunatic@0.12.1::logging::LoggingProcess");
/// ```
///
/// The process name can be overridden with the `#[lunatic(process_name = "...")]` attribute.
///
/// ```ignore
/// #[derive(ProcessName)]
/// #[lunatic(process_name = "global_logging_process")]
/// struct LoggingProcess;
///
/// assert_eq!(LoggingProcess.process_name(), "global_logging_process");
/// ```
#[proc_macro_derive(ProcessName, attributes(lunatic))]
pub fn process_name(input: TokenStream) -> TokenStream {
    let process_name_derive = parse_macro_input!(input as ProcessNameDerive);
    process_name_derive.to_token_stream().into()
}

fn token_stream_with_error(mut tokens: TokenStream, error: syn::Error) -> TokenStream {
    tokens.extend(TokenStream::from(error.into_compile_error()));
    tokens
}
