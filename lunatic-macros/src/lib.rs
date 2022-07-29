#[allow(unused_extern_crates)]
extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;

mod abstract_process;
use abstract_process::AbstractProcessTransformer;

/// Marks the main function to be executed by the lunatic runtime as the root process.
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

/// Add [`AbstractProcess`] behavior to the given struct implementation with minimum
/// boilerplate code.
///
/// - Use `#[init]`, `#[terminate]`, and `#[handle_link_trapped]` attributes to
/// specify methods for implementing [`AbstractProcess`].
/// - Use `#[handle_message]` and `#[handle_request]` attributes to specify
/// message and request handlers.
///
/// Specifying message types is unnecessary because the macro will create wrapper
/// types for messages on all handlers. Handlers can take arbitrary number of
/// parameters and invocating them works the same as directly calling the method
/// on the struct without spawning it as a process.
///
/// A trait is generated and defaults to private and follows the name of your type with
/// `Handler` added as a suffix. To rename or change the visibility of the generated
/// trait, you can use the `trait_name` and `visbility` arguments with
/// `#[abstract_process(trait_name = "MyHandler", visibility = pub)]`.
///
/// # Examples
///
/// ```ignore
/// use lunatic::{
///     abstract_process,
///     process::{Message, ProcessRef, Request, StartProcess},
///     Tag,
/// };
///
/// struct Counter(u32);
///
/// #[abstract_process]
/// impl Counter {
///     #[init]
///     fn init(_: ProcessRef<Self>, start: u32) -> Self {
///         Self(start)
///     }
///
///     #[terminate]
///     fn terminate(self) {
///         println!("Shutdown process");
///     }
///
///     #[handle_link_trapped]
///     fn handle_link_trapped(&self, tag: Tag) {
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
/// }
///
///
/// let counter = Counter::start(5, None);
/// counter.increment();
/// assert_eq!(counter.count(), 6);
/// ```
///
/// A more complicated example
///
/// ```ignore
/// use lunatic::{
///     abstract_process,
///     process::{Message, ProcessRef, Request, StartProcess},
/// }
///
/// struct A;
///
/// #[derive(serde::Serialize, serde::Deserialize)]
/// struct Person {
///     name: String,
///     age: u16,
/// }
///
/// #[abstract_process(trait_name = "AHandler", visibility = pub)]
/// impl A {
///     #[init]
///     fn init(_: ProcessRef<Self>, _: ()) -> A {
///         A
///     }
///
///     #[hanlde_message]
///     fn multiple_arguments(&self, a: u8, (b, c): (bool, char)) {
///         assert_eq!(a, 5);
///         assert_eq!(b, false);
///         assert_eq!(c, 'a');
///     }
///
///     #[handle_request]
///     fn unpack_struct(&self, Person { name, age }: Person) -> String {
///         assert_eq!(name, "Mark");
///         assert_eq!(age, 5);
///         self.create_greeting(name)
///     }
///
///     fn create_greeting(&self, name: String) {
///         format!("Hi {}!", name)
///     }
/// }
///
///
/// let a = A::start_link((), None);
///
/// a.multiple_arguments(5, (false, 'a'));
///
/// let person = Person {
///     name: "Mark".to_owned(),
///     age: 4,
/// };
///
/// let greeting = a.unpack_struct(person);
/// assert_eq!(greeting, "Hi Mark!");
/// ```
///
/// [`AbstractProcess`]: process/trait.AbstractProcess.html
#[proc_macro_attribute]
pub fn abstract_process(args: TokenStream, item: TokenStream) -> TokenStream {
    let args: abstract_process::Args = match syn::parse(args) {
        Ok(args) => args,
        Err(e) => return token_stream_with_error(item, e),
    };
    match syn::parse(item.clone()) {
        Ok(it) => AbstractProcessTransformer::new().transform(args, it).into(),
        Err(e) => token_stream_with_error(item, e),
    }
}

fn token_stream_with_error(mut tokens: TokenStream, error: syn::Error) -> TokenStream {
    tokens.extend(TokenStream::from(error.into_compile_error()));
    tokens
}
