use convert_case::{Case, Casing};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::ImplItem::Method;
use syn::Token;

/// Transform and expand the `abstract_process` macro
#[derive(Default)]
pub struct AbstractProcessTransformer {
    /// impl type
    impl_type: Option<syn::Path>,
    impl_type_name: Option<syn::Ident>,
    impl_type_generics: syn::Generics,
    /// impl type macros
    impl_type_attrs: Vec<syn::Attribute>,
    /// implmentation of trait `AbstractProcess`
    ap_impls: AbstractProcessImpls,
    /// Type impl block that is received by the macro
    type_impls: TypeImpls,
    /// Wrapper methods for send and request
    handler_wrappers: HandlerWrappers,
    /// Delayed message builder for invoking `send_after`
    msg_builder_methods: Vec<TokenStream>,
    /// Timeout request builder for invoking `request_timeout`
    req_builder_methods: Vec<TokenStream>,
    /// message (message, request, and response) struct definitions
    message_structs: Vec<TokenStream>,
    /// impl blocks for MessageHandler and RequestHandler
    handler_impls: Vec<TokenStream>,
    /// compiler errors
    errors: Vec<TokenStream>,
}

impl AbstractProcessTransformer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn transform(&mut self, args: Args, impl_block: syn::ItemImpl) -> TokenStream {
        self.extract(args, impl_block);
        self.render()
    }

    fn extract(&mut self, args: Args, impl_block: syn::ItemImpl) {
        let impl_block_span = impl_block.span();

        self.impl_type_attrs = impl_block.attrs;
        self.impl_type = match *impl_block.self_ty {
            syn::Type::Path(p) => Some(p.path),
            _ => unreachable!("The given impl type is not supported"),
        };
        self.impl_type_name = self
            .impl_type
            .as_ref()
            .and_then(|t| t.segments.last())
            .map(|s| s.ident.clone())
            .or_else(|| {
                let err = syn::Error::new(
                    impl_block_span,
                    "`impl` type is not supported by `#[abstract_process]`",
                )
                .to_compile_error();
                self.errors.push(err);
                // create temporary to silence error from invalid syntax
                Some(syn::Ident::new("__Placeholder", Span::call_site()))
            });
        self.impl_type_generics = impl_block.generics;

        self.handler_wrappers.trait_visibility = args.visibility;
        self.handler_wrappers.trait_name = args
            .trait_name
            .map(|trait_name| format_ident!("{}", trait_name.value()))
            .or_else(|| {
                self.impl_type_name
                    .as_ref()
                    .map(|n| format_ident!("{}Handler", n))
            });

        for item in &impl_block.items {
            match item {
                Method(method) if method.has_tag("init") => self.extract_init(method),
                Method(method) if method.has_tag("terminate") => self.extract_terminate(method),
                Method(method) if method.has_tag("handle_link_trapped") => {
                    self.extract_handle_link_trapped(method)
                }
                Method(method) if method.has_tag("handle_message") => {
                    self.extract_handle_message(method);
                }
                Method(method) if method.has_tag("handle_request") => {
                    self.extract_handle_request(method);
                }
                _ => {
                    self.type_impls.skipped_items.push(quote! { #item });
                }
            }
        }

        // ensure init exists
        if self.ap_impls.init.is_none() {
            let err = syn::Error::new(
                impl_block_span,
                "Must implement the `init` method marked with `#[init]`",
            )
            .to_compile_error();
            self.errors.push(err);
        }
    }

    fn render(&self) -> TokenStream {
        let errors = &self.errors;
        let impl_type = &self.impl_type;
        let (impl_generics, ty_generics, where_clause) = &self.impl_type_generics.split_for_impl();
        let AbstractProcessImpls {
            init: init_impl,
            terminate: terminate_impl,
            handle_link_trapped: handle_link_trapped_impl,
        } = &self.ap_impls;
        let TypeImpls {
            terminate,
            handle_link_trapped,
            skipped_items,
        } = &self.type_impls;
        let message_structs = &self.message_structs;
        let impl_attrs = &self.impl_type_attrs;
        let handler_impls = &self.handler_impls;
        let HandlerWrappers {
            trait_visibility: handler_visibility,
            trait_name: handler_trait,
            trait_defs: handler_wrapper_defs,
            trait_impls: handler_wrapper_impls,
        } = &self.handler_wrappers;
        let msg_builder_methods = &self.msg_builder_methods;
        let req_builder_methods = &self.req_builder_methods;

        let msg_builder_struct = self
            .impl_type_name
            .as_ref()
            .map(|i| format_ident!("{}MsgBuilder", i));
        let req_builder_struct = self
            .impl_type_name
            .as_ref()
            .map(|i| format_ident!("{}ReqBuilder", i));

        quote! {
            #(#errors)*

            #(#message_structs)*

            #(#impl_attrs)*
            impl #impl_generics #impl_type #where_clause {
                #terminate
                #handle_link_trapped
                #(#skipped_items)*
            }

            impl #impl_generics lunatic::process::AbstractProcess for #impl_type #where_clause {
                type State = #impl_type;
                #init_impl
                #terminate_impl
                #handle_link_trapped_impl
            }

            #(#handler_impls)*

            #handler_visibility trait #handler_trait #ty_generics #where_clause {
                #(#handler_wrapper_defs)*

                /// set a delay before sending the message
                fn after(&self, duration: std::time::Duration) -> #msg_builder_struct #ty_generics;

                /// set a timeout for the request
                fn with_timeout(&self, duration: std::time::Duration) -> #req_builder_struct #ty_generics;
            }

            impl #impl_generics #handler_trait #ty_generics for lunatic::process::ProcessRef<#impl_type>
            #where_clause {
                #(#handler_wrapper_impls)*

                fn after(&self, duration: std::time::Duration) -> #msg_builder_struct #ty_generics {
                    #msg_builder_struct::new(duration, self.clone())
                }

                fn with_timeout(&self, duration: std::time::Duration) -> #req_builder_struct #ty_generics {
                    #req_builder_struct::new(duration, self.clone())
                }
            }

            #handler_visibility struct #msg_builder_struct #ty_generics #where_clause {
                duration: std::time::Duration,
                process_ref: lunatic::process::ProcessRef<#impl_type>,
            }

            impl #impl_generics #msg_builder_struct #ty_generics #where_clause {
                fn new(duration: std::time::Duration, process_ref: ProcessRef<#impl_type>) -> Self {
                    Self { duration, process_ref }
                }

                #(#handler_visibility #msg_builder_methods)*
            }

            #handler_visibility struct #req_builder_struct #ty_generics #where_clause {
                duration: std::time::Duration,
                process_ref: lunatic::process::ProcessRef<#impl_type>,
            }

            impl #impl_generics #req_builder_struct #ty_generics #where_clause {
                fn new(duration: std::time::Duration, process_ref: ProcessRef<#impl_type>) -> Self {
                    Self { duration, process_ref }
                }

                #(#handler_visibility #req_builder_methods)*
            }
        }
    }

    fn extract_init(&mut self, method: &syn::ImplItemMethod) {
        if self.ap_impls.init.is_some() {
            let err = syn::Error::new(
                method.sig.ident.span(),
                "Only one method can be marked with `#[init]` macro",
            )
            .into_compile_error();
            self.errors.push(err);
            return;
        }
        let attrs = method
            .attrs
            .iter()
            .filter(|attr| !attr.path.is_ident("init"));

        let sig = &method.sig;
        // ensure function name is init
        let ident = &sig.ident;
        let error = if sig.ident != "init" {
            Some(
                syn::Error::new(
                    sig.ident.span(),
                    "Invalid method signature. Method name must be `init`.",
                )
                .into_compile_error(),
            )
        } else {
            None
        };

        let arg_type = if let Some(syn::FnArg::Typed(arg)) = sig.inputs.last() {
            &arg.ty
        } else {
            unreachable!("Other cases will be caught prior to this at syn::parse")
        };
        let func_args = &sig.inputs;
        let block = &method.block;

        self.ap_impls.init = Some(quote! {
            #error

            type Arg = #arg_type;

            #(#attrs)*
            fn #ident(#func_args) -> Self::State #block
        });
    }

    fn extract_terminate(&mut self, method: &syn::ImplItemMethod) {
        if self.type_impls.terminate.is_some() {
            let err = syn::Error::new(
                method.sig.ident.span(),
                "Only one method can be marked with `#[terminate]` macro",
            )
            .into_compile_error();
            self.errors.push(err);
        }
        let sig = &method.sig;
        // ensure function name is terminate
        let ident = &sig.ident;
        let error = if sig.ident != "terminate" {
            Some(
                syn::Error::new(
                    sig.ident.span(),
                    "Invalid method signature. Method name must be `terminate`.",
                )
                .into_compile_error(),
            )
        } else {
            None
        };

        let self_arg = &sig.inputs;
        let block = &method.block;

        self.type_impls.terminate = Some(quote! {
            #error

            fn #ident(#self_arg) #block
        });
        self.ap_impls.terminate = Some(quote! {
            fn terminate(state: Self::State) {
                state.terminate()
            }
        });
    }

    fn extract_handle_link_trapped(&mut self, method: &syn::ImplItemMethod) {
        if self.type_impls.handle_link_trapped.is_some() {
            let err = syn::Error::new(
                method.sig.ident.span(),
                "Only one method can be marked with `#[handle_link_trapped]` macro",
            )
            .into_compile_error();
            self.errors.push(err);
        }
        let sig = &method.sig;
        // ensure function name is handle_link_trapped
        let ident = &sig.ident;
        let error = if sig.ident != "handle_link_trapped" {
            Some(
                syn::Error::new(
                    sig.ident.span(),
                    "Invalid method signature. Method name must be `handle_link_trapped`.",
                )
                .into_compile_error(),
            )
        } else {
            None
        };

        let self_arg = &sig.inputs;
        let block = &method.block;

        self.type_impls.handle_link_trapped = Some(quote! {
            #error

            fn #ident(#self_arg) #block
        });
        self.ap_impls.handle_link_trapped = Some(quote! {
            fn handle_link_trapped(state: &mut Self::State, tag: Tag) {
                state.handle_link_trapped(tag);
            }
        });
    }

    fn extract_handle_message(&mut self, method: &syn::ImplItemMethod) {
        let mut method = method.clone();
        method
            .attrs
            .retain(|attr| !attr.path.is_ident("handle_message"));
        let attrs = &method.attrs;

        let HandlerComponents {
            fn_ident,
            message_type,
            handler_args,
            handler_arg_names,
            handler_arg_types,
            message_destructuring,
        } = self.parse_handler_input(&method);

        let ident = &self.impl_type.clone().unwrap();

        let (impl_generics, ty_generics, where_clause) = &self.impl_type_generics.split_for_impl();

        let (msg_phantom, arg_phantom) = if self.impl_type_generics.params.is_empty() {
            (None, None)
        } else {
            (
                Some(quote! { std::marker::PhantomData #ty_generics, }),
                Some(quote! { std::marker::PhantomData, }),
            )
        };

        self.message_structs.push(quote! {
            #[derive(serde::Serialize, serde::Deserialize)]
            struct #message_type #ty_generics (
                #msg_phantom
                #(#handler_arg_types),*
            );
        });
        self.handler_impls.push(quote! {
            #(#attrs)*
            impl #impl_generics lunatic::process::MessageHandler<#message_type #ty_generics>
            for #ident #where_clause {
                fn handle(state: &mut Self::State, message: #message_type #ty_generics) {
                    state.#fn_ident(#(#message_destructuring),*)
                }
            }
        });
        self.handler_wrappers.trait_defs.push(quote! {
            #(#attrs)*
            fn #fn_ident(&self, #(#handler_args),*);
        });
        self.handler_wrappers.trait_impls.push(quote! {
            #(#attrs)*
            fn #fn_ident(&self, #(#handler_args),*) {
                use lunatic::process::Message;
                let msg = #message_type(#arg_phantom #(#handler_arg_names),*);
                self.send(msg);
            }
        });
        self.msg_builder_methods.push(quote! {
            #(#attrs)*
            fn #fn_ident(&self, #(#handler_args),*) {
                use lunatic::process::Message;
                let msg = #message_type(#arg_phantom #(#handler_arg_names),*);
                self.process_ref.send_after(msg, self.duration);
            }
        });
        self.type_impls.skipped_items.push(quote! { #method });
    }

    fn extract_handle_request(&mut self, method: &syn::ImplItemMethod) {
        let mut method = method.clone();
        method
            .attrs
            .retain(|attr| !attr.path.is_ident("handle_request"));
        let attrs = &method.attrs;

        let HandlerComponents {
            fn_ident,
            message_type,
            handler_args,
            handler_arg_names,
            handler_arg_types,
            message_destructuring,
        } = self.parse_handler_input(&method);

        let ident = &self.impl_type.clone().unwrap();

        let (impl_generics, ty_generics, where_clause) = &self.impl_type_generics.split_for_impl();

        let (msg_phantom, arg_phantom) = if self.impl_type_generics.params.is_empty() {
            (None, None)
        } else {
            (
                Some(quote! { std::marker::PhantomData #ty_generics, }),
                Some(quote! { std::marker::PhantomData, }),
            )
        };

        let response_type = match &method.sig.output {
            syn::ReturnType::Type(_, ty) => quote! { #ty },
            syn::ReturnType::Default => {
                quote! { () }
            }
        };

        self.message_structs.push(quote! {
            #[derive(serde::Serialize, serde::Deserialize)]
            struct #message_type #ty_generics (
                #msg_phantom
                #(#handler_arg_types),*
            );
        });
        self.handler_impls.push(quote! {
            #(#attrs)*
            impl #impl_generics lunatic::process::RequestHandler<#message_type #ty_generics>
            for #ident #where_clause {
                type Response = #response_type;
                fn handle(state: &mut Self::State, message: #message_type #ty_generics) -> #response_type {
                    state.#fn_ident(#(#message_destructuring),*)
                }
            }
        });
        self.handler_wrappers.trait_defs.push(quote! {
            #(#attrs)*
            fn #fn_ident(&self, #(#handler_args),*) -> #response_type;
        });
        self.handler_wrappers.trait_impls.push(quote! {
            #(#attrs)*
            fn #fn_ident(&self, #(#handler_args),*) -> #response_type {
                use lunatic::process::Request;
                let req = #message_type(#arg_phantom #(#handler_arg_names),*);
                self.request(req)
            }
        });
        self.req_builder_methods.push(quote! {
            #(#attrs)*
            fn #fn_ident(&self, #(#handler_args),*) -> lunatic::MailboxResult<#response_type> {
                use lunatic::process::Request;
                let req = #message_type(#arg_phantom #(#handler_arg_names),*);
                self.process_ref.request_timeout(req, self.duration)
            }
        });
        self.type_impls.skipped_items.push(quote! { #method });
    }

    fn parse_handler_input(&self, item: &syn::ImplItemMethod) -> HandlerComponents {
        let sig = &item.sig;
        let fn_ident = &sig.ident;
        let message_type = proc_macro2::Ident::new(
            &format!("__MsgWrap{}", fn_ident.to_string().to_case(Case::Pascal)),
            Span::call_site(),
        );

        // wrap message types
        let mut handler_args: Vec<TokenStream> = vec![];
        let mut handler_arg_names: Vec<syn::Ident> = vec![];
        let mut handler_arg_types: Vec<syn::Type> = vec![];

        for (i, arg) in sig.inputs.iter().skip(1).enumerate() {
            // take apart the argument identifiers and their types
            let (ident, ty) = match arg {
                syn::FnArg::Typed(arg) => {
                    let ident = match *arg.pat.clone() {
                        // replace patterns with generated identifiers to prevent syntax error
                        syn::Pat::Ident(pat_ident) => pat_ident.ident,
                        _ => proc_macro2::Ident::new(&format!("__arg_{}", i), Span::call_site()),
                    };
                    (ident, *arg.ty.clone())
                }
                _ => unreachable!("Second arguement, if exist, will always be typed"),
            };
            // rebuild args list
            handler_args.push(quote! { #ident: #ty });
            handler_arg_names.push(ident);
            handler_arg_types.push(ty);
        }
        let has_generics = !self.impl_type_generics.params.is_empty();
        let message_destructuring = (0..handler_arg_types.len())
            .map(|i| {
                // offset for PhantomData
                let offset = if has_generics { 1 } else { 0 };
                let i = proc_macro2::Literal::usize_unsuffixed(i + offset);
                quote! { message.#i }
            })
            .collect();

        HandlerComponents {
            fn_ident: fn_ident.clone(),
            message_type,
            handler_args,
            handler_arg_names,
            handler_arg_types,
            message_destructuring,
        }
    }
}

#[derive(Default)]
pub struct Args {
    trait_name: Option<syn::LitStr>,
    visibility: Option<syn::Visibility>,
}

impl Args {
    fn parse_arg(&mut self, input: ParseStream) -> syn::Result<()> {
        if input.is_empty() {
            return Ok(());
        }

        let ident: syn::Ident = input.parse()?;
        let _: syn::Token![=] = input.parse()?;
        if ident == "trait_name" {
            if self.trait_name.is_some() {
                return Err(syn::Error::new(
                    ident.span(),
                    "trait name already specified",
                ));
            }

            self.trait_name = Some(input.parse()?);
        } else if ident == "visibility" {
            if self.visibility.is_some() {
                return Err(syn::Error::new(
                    ident.span(),
                    "visibility already specified",
                ));
            }

            self.visibility = Some(input.parse()?);
        } else {
            return Err(syn::Error::new(ident.span(), "unknown argument"));
        }

        Ok(())
    }
}

impl Parse for Args {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut args = Args::default();
        while !input.is_empty() {
            args.parse_arg(input)?;
            let _: Option<Token![,]> = input.parse()?;
        }
        Ok(args)
    }
}

struct HandlerComponents {
    fn_ident: syn::Ident,
    message_type: syn::Ident,
    handler_args: Vec<TokenStream>,
    handler_arg_names: Vec<syn::Ident>,
    handler_arg_types: Vec<syn::Type>,
    message_destructuring: Vec<TokenStream>,
}

#[derive(Default)]
struct TypeImpls {
    terminate: Option<TokenStream>,
    handle_link_trapped: Option<TokenStream>,
    skipped_items: Vec<TokenStream>,
}

#[derive(Default)]
struct AbstractProcessImpls {
    init: Option<TokenStream>,
    terminate: Option<TokenStream>,
    handle_link_trapped: Option<TokenStream>,
}

#[derive(Default)]
struct HandlerWrappers {
    trait_visibility: Option<syn::Visibility>,
    trait_name: Option<syn::Ident>,
    trait_defs: Vec<TokenStream>,
    trait_impls: Vec<TokenStream>,
}

trait HasTag {
    fn has_tag(&self, tag: &str) -> bool;
}

impl HasTag for syn::ImplItemMethod {
    fn has_tag(&self, tag: &str) -> bool {
        self.attrs.iter().any(|attr| attr.path.is_ident(tag))
    }
}
