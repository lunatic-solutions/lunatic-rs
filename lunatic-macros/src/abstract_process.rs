use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::Token;

/// AbstractProcess macro.
pub struct AbstractProcess {
    /// Arguments passed to the `#[abstract_process(...)]` macro.
    args: Args,
    /// Original impl item.
    item_impl: syn::ItemImpl,
    /// Arg type in abstract process implementation.
    arg_ty: syn::Type,
    /// Init method.
    init: syn::ImplItemMethod,
    /// Terminate method.
    terminate: Option<syn::ImplItemMethod>,
    /// Handle link trapped method.
    handle_link_trapped: Option<syn::ImplItemMethod>,
    /// Message handler methods.
    message_handlers: Vec<syn::ImplItemMethod>,
    /// Request handler methods.
    request_handlers: Vec<syn::ImplItemMethod>,
    /// Ident of the message builder struct.
    message_builder_ident: syn::Ident,
    /// Ident of the request builder struct.
    request_builder_ident: syn::Ident,
    /// Ident of the handler trait.
    handler_trait_ident: syn::Ident,
}

impl AbstractProcess {
    /// Parses and validates an impl statement and attributes.
    pub fn new(args: proc_macro::TokenStream, item: proc_macro::TokenStream) -> syn::Result<Self> {
        let args: Args = syn::parse(args)?;
        let mut item_impl: syn::ItemImpl = syn::parse(item)?;
        let self_ident = match *item_impl.self_ty {
            syn::Type::Path(ref ty_path) => match ty_path.path.segments.last() {
                Some(segment) => segment.ident.clone(),
                None => {
                    return Err(syn::Error::new(
                        item_impl.self_ty.span(),
                        "type is not supported",
                    ))
                }
            },
            _ => {
                return Err(syn::Error::new(
                    item_impl.self_ty.span(),
                    "type is not supported",
                ))
            }
        };
        let (init, terminate, handle_link_trapped, message_handlers, request_handlers) = item_impl
            .items
            .clone()
            .into_iter()
            .enumerate()
            .filter_map(|(i, item)| match item {
                syn::ImplItem::Method(impl_item_method) => Some((i, impl_item_method)),
                _ => None,
            })
            .filter_map(|(i, mut impl_item_method)| {
                let (j, item_attr) =
                    impl_item_method
                        .attrs
                        .iter()
                        .enumerate()
                        .find_map(|(i, attr)| {
                            attr.path
                                .get_ident()
                                .map(|ident| ident.to_string())
                                .and_then(|ident_string| ItemAttr::from_str(&ident_string))
                                .map(|item_attr| (i, item_attr))
                        })?;
                // We found an attribute, we should remove it from the original item_impl
                impl_item_method.attrs.remove(j);
                if let syn::ImplItem::Method(impl_item_method) = item_impl.items.get_mut(i).unwrap()
                {
                    impl_item_method.attrs.remove(j);
                }

                Some((item_attr, impl_item_method))
            })
            .fold(
                Ok((None, None, None, Vec::new(), Vec::new())),
                |acc, (item_attr, impl_item_method)| {
                    let (
                        mut init,
                        mut terminate,
                        mut handle_link_trapped,
                        mut message_handlers,
                        mut request_handlers,
                    ) = acc?;

                    match item_attr {
                        ItemAttr::Init => {
                            if init.is_some() {
                                return Err(syn::Error::new(
                                    impl_item_method.sig.ident.span(),
                                    "init method already defined",
                                ));
                            }

                            init = Some(impl_item_method);
                        }
                        ItemAttr::Terminate => {
                            if terminate.is_some() {
                                return Err(syn::Error::new(
                                    impl_item_method.sig.ident.span(),
                                    "terminate method already defined",
                                ));
                            }

                            terminate = Some(impl_item_method);
                        }
                        ItemAttr::HandleLinkTrapped => {
                            if handle_link_trapped.is_some() {
                                return Err(syn::Error::new(
                                    impl_item_method.sig.ident.span(),
                                    "handle_link_trapped method already defined",
                                ));
                            }

                            handle_link_trapped = Some(impl_item_method);
                        }
                        ItemAttr::HandleMessage => {
                            message_handlers.push(impl_item_method);
                        }
                        ItemAttr::HandleRequest => {
                            request_handlers.push(impl_item_method);
                        }
                    }

                    Ok((
                        init,
                        terminate,
                        handle_link_trapped,
                        message_handlers,
                        request_handlers,
                    ))
                },
            )?;

        let init =
            init.ok_or_else(|| syn::Error::new(item_impl.self_ty.span(), "missing init method"))?;
        let arg_ty = match init
            .sig
            .inputs
            .last()
            .ok_or_else(|| syn::Error::new(init.sig.span(), "init must take 2 arguments"))?
        {
            syn::FnArg::Receiver(_) => {
                return Err(syn::Error::new(init.sig.span(), "init cannot take `&self`"))
            }
            syn::FnArg::Typed(typed_arg) => *typed_arg.ty.clone(),
        };

        let message_builder_ident = format_ident!("{}MsgBuilder", self_ident);
        let request_builder_ident = format_ident!("{}ReqBuilder", self_ident);
        let handler_trait_ident = args
            .trait_name
            .as_ref()
            .map(|trait_name| format_ident!("{}", trait_name.value()))
            .unwrap_or_else(|| format_ident!("{}Handler", self_ident));

        Ok(AbstractProcess {
            args,
            item_impl,
            arg_ty,
            init,
            terminate,
            handle_link_trapped,
            message_handlers,
            request_handlers,
            message_builder_ident,
            request_builder_ident,
            handler_trait_ident,
        })
    }

    /// Expands macro.
    pub fn expand(&self) -> TokenStream {
        let handler_wrappers = self.expand_handler_wrappers();
        let original_impl = self.expand_original_impl();
        let impl_abstract_process = self.expand_impl_abstract_process();
        let message_handler_impls = self.expand_message_handler_impls();
        let request_handler_impls = self.expand_request_handler_impls();
        let handler_trait = self.expand_handler_trait();
        let impl_handler_trait = self.expand_impl_handler_trait();
        let message_builders = self.expand_builders();

        quote! {
            #handler_wrappers
            #original_impl
            #impl_abstract_process
            #message_handler_impls
            #request_handler_impls
            #handler_trait
            #impl_handler_trait
            #message_builders
        }
    }

    /// Expands handler wrapper structs.
    ///
    /// ```ignore
    /// __MsgWrapFoo(Param1, Param2);
    /// __MsgWrapBar(Param1, Param2);
    /// ```
    fn expand_handler_wrappers(&self) -> TokenStream {
        let wrappers = self
            .message_handlers
            .iter()
            .chain(self.request_handlers.iter())
            .map(|impl_item_method| self.expand_handler_wrapper(impl_item_method));

        quote! {
            #( #wrappers )*
        }
    }

    /// Expands a single handler wrapper struct.
    ///
    /// ```ignore
    /// __MsgWrap(Param1, Param2);
    /// ```
    fn expand_handler_wrapper(&self, impl_item_method: &syn::ImplItemMethod) -> TokenStream {
        let ident = Self::handler_wrapper_ident(&impl_item_method.sig.ident);
        let (_, ty_generics, _) = self.item_impl.generics.split_for_impl();
        let fields = filter_typed_args(impl_item_method.sig.inputs.iter()).map(|field| &*field.ty);
        let phantom_field = if !self.item_impl.generics.params.is_empty() {
            Some(quote! { std::marker::PhantomData #ty_generics, })
        } else {
            None
        };

        quote! {
            #[derive(serde::Serialize, serde::Deserialize)]
            struct #ident #ty_generics (
                #phantom_field
                #( #fields ),*
            );
        }
    }

    /// Expands the original implementation written.
    fn expand_original_impl(&self) -> TokenStream {
        let syn::ItemImpl {
            attrs,
            defaultness,
            unsafety,
            impl_token,
            generics,
            self_ty,
            items,
            ..
        } = &self.item_impl;
        let (impl_generics, _ty_generics, where_clause) = generics.split_for_impl();

        quote! {
            #( #attrs )*
            #defaultness #unsafety #impl_token #impl_generics #self_ty #where_clause {
                #( #items )*
            }
        }
    }

    /// Expands the implementation for abstract process.
    fn expand_impl_abstract_process(&self) -> TokenStream {
        let syn::ItemImpl {
            generics, self_ty, ..
        } = &self.item_impl;

        let (impl_generics, _ty_generics, where_clause) = generics.split_for_impl();
        let arg_ty = &self.arg_ty;

        let init_impl = self.expand_init_impl();
        let terminate_impl = self.expand_terminate_impl();
        let handle_link_trapped_impl = self.expand_handle_link_trapped_impl();

        quote! {
            impl #impl_generics lunatic::process::AbstractProcess for #self_ty #where_clause {
                type State = #self_ty;
                type Arg = #arg_ty;

                #init_impl
                #terminate_impl
                #handle_link_trapped_impl
            }
        }
    }

    /// Expands the `init` method in the abstract process implementation.
    fn expand_init_impl(&self) -> TokenStream {
        let ident = &self.init.sig.ident;
        let arg_ty = &self.arg_ty;

        quote! {
            fn init(this: ProcessRef<Self>, arg: #arg_ty) -> Self::State {
                Self::#ident(this, arg)
            }
        }
    }

    /// Expands the `terminate` method in the abstract process implementation.
    fn expand_terminate_impl(&self) -> TokenStream {
        self.terminate
            .as_ref()
            .map(|terminate| {
                let ident = &terminate.sig.ident;

                quote! {
                    fn terminate(state: Self::State) {
                        state.#ident()
                    }
                }
            })
            .unwrap_or_default()
    }

    /// Expands the `handle_link_trapped` method in the abstract process
    /// implementation.
    fn expand_handle_link_trapped_impl(&self) -> TokenStream {
        self.handle_link_trapped
            .as_ref()
            .map(|handle_link_trapped| {
                let ident = &handle_link_trapped.sig.ident;

                quote! {
                    fn handle_link_trapped(state: &mut Self::State, tag: lunatic::Tag) {
                        state.#ident(tag);
                    }
                }
            })
            .unwrap_or_default()
    }

    /// Expands the `MessageHandler` implementations for the message hander
    /// wrapper types.
    fn expand_message_handler_impls(&self) -> TokenStream {
        let message_handler_impls = self.message_handlers.iter().map(|message_handler| {
            let syn::ImplItemMethod {
                attrs,
                sig,
                ..
            } = message_handler;
            let self_ty = &self.item_impl.self_ty;
            let message_type = Self::handler_wrapper_ident(&sig.ident);
            let fn_ident = &sig.ident;
            let (impl_generics, ty_generics, where_clause) = self.item_impl.generics.split_for_impl();
            let args = filter_typed_args(sig.inputs.iter());
            let offset = if self.item_impl.generics.params.is_empty() { 0 } else { 1 };
            let message_fields = (offset..args.count() + offset).map(|i| {
                let i = proc_macro2::Literal::usize_unsuffixed(i);
                quote! { message. #i }
            });

            quote! {
                #( #attrs )*
                impl #impl_generics lunatic::process::MessageHandler<#message_type #ty_generics> for #self_ty #where_clause {
                    fn handle(state: &mut Self::State, message: #message_type #ty_generics) {
                        state.#fn_ident(#( #message_fields ),*)
                    }
                }
            }
        });

        quote! {
            #( #message_handler_impls )*
        }
    }

    /// Expands the `RequestHandler` implementations for the request hander
    /// wrapper types.
    fn expand_request_handler_impls(&self) -> TokenStream {
        let request_handler_impls = self.request_handlers.iter().map(|request_handler| {
            let syn::ImplItemMethod {
                attrs,
                sig,
                ..
            } = request_handler;
            let self_ty = &self.item_impl.self_ty;
            let request_type = Self::handler_wrapper_ident(&sig.ident);
            let response_type = match &sig.output {
                syn::ReturnType::Type(_, ty) => quote! { #ty },
                syn::ReturnType::Default => {
                    quote! { () }
                }
            };
            let fn_ident = &sig.ident;
            let (impl_generics, ty_generics, where_clause) = self.item_impl.generics.split_for_impl();
            let args = filter_typed_args(sig.inputs.iter());
            let offset = if self.item_impl.generics.params.is_empty() { 0 } else { 1 };
            let request_fields = (offset..args.count() + offset).map(|i| {
                let i = proc_macro2::Literal::usize_unsuffixed(i);
                quote! { request. #i }
            });

            quote! {
                #( #attrs )*
                impl #impl_generics lunatic::process::RequestHandler<#request_type #ty_generics> for #self_ty #where_clause {
                    type Response = #response_type;

                    fn handle(state: &mut Self::State, request: #request_type #ty_generics) -> Self::Response {
                        state.#fn_ident(#( #request_fields ),*)
                    }
                }
            }
        });

        quote! {
            #( #request_handler_impls )*
        }
    }

    /// Expands the new `Handler` trait.
    fn expand_handler_trait(&self) -> TokenStream {
        let Self {
            args,
            item_impl,
            message_handlers,
            request_handlers,
            message_builder_ident,
            request_builder_ident,
            handler_trait_ident,
            ..
        } = self;
        let vis = &args.visibility;
        let (_impl_generics, ty_generics, where_clause) = item_impl.generics.split_for_impl();

        let message_handler_defs = message_handlers
            .iter()
            .map(HandlerStructure::from_handler)
            .map(|handler| {
                let HandlerStructure {
                    attrs,
                    ident,
                    generics,
                    args,
                    ..
                } = handler;

                quote! {
                    #( #attrs )*
                    fn #ident #generics (&self #(, #args )*);
                }
            });

        let request_handler_defs = request_handlers
            .iter()
            .map(HandlerStructure::from_handler)
            .map(|handler| {
                let HandlerStructure {
                    attrs,
                    ident,
                    generics,
                    args,
                    return_ty,
                    ..
                } = handler;

                quote! {
                    #( #attrs )*
                    fn #ident #generics (&self #(, #args )*) -> #return_ty;
                }
            });

        quote! {
            #vis trait #handler_trait_ident #ty_generics #where_clause {
                #( #message_handler_defs )*
                #( #request_handler_defs )*

                /// Set a delay before sending the message.
                fn after(&self, duration: std::time::Duration) -> #message_builder_ident #ty_generics;

                /// Set a timeout for the request.
                fn with_timeout(&self, duration: std::time::Duration) -> #request_builder_ident #ty_generics;
            }
        }
    }

    /// Expands the implementation of the `Handler` trait.
    fn expand_impl_handler_trait(&self) -> TokenStream {
        let Self {
            item_impl,
            message_handlers,
            request_handlers,
            message_builder_ident,
            request_builder_ident,
            handler_trait_ident,
            ..
        } = self;
        let self_ty = &item_impl.self_ty;
        let (impl_generics, ty_generics, where_clause) = item_impl.generics.split_for_impl();
        let arg_phantom = if !item_impl.generics.params.is_empty() {
            Some(quote! { std::marker::PhantomData, })
        } else {
            None
        };

        let message_handler_impls = message_handlers
            .iter()
            .map(HandlerStructure::from_handler)
            .map(|handler| {
                let HandlerStructure {
                    attrs,
                    ident,
                    generics,
                    args,
                    message_type,
                    handler_args,
                    ..
                } = handler;

                quote! {
                    #( #attrs )*
                    fn #ident #generics (&self #(, #args )*) {
                        use lunatic::process::Message;
                        let msg = #message_type(#arg_phantom #( #handler_args ),*);
                        self.send(msg);
                    }
                }
            });

        let request_handler_impls = request_handlers
            .iter()
            .map(HandlerStructure::from_handler)
            .map(|handler| {
                let HandlerStructure {
                    attrs,
                    ident,
                    generics,
                    args,
                    return_ty,
                    message_type,
                    handler_args,
                } = handler;

                quote! {
                    #( #attrs )*
                    fn #ident #generics (&self #(, #args )*) -> #return_ty {
                        use lunatic::process::Request;
                        let req = #message_type(#arg_phantom #( #handler_args ),*);
                        self.request(req)
                    }
                }
            });

        quote! {
            impl #impl_generics #handler_trait_ident #ty_generics for lunatic::process::ProcessRef<#self_ty> #where_clause {
                #( #message_handler_impls )*
                #( #request_handler_impls )*

                fn after(&self, duration: std::time::Duration) -> #message_builder_ident #ty_generics {
                    #message_builder_ident::new(duration, self.clone())
                }

                fn with_timeout(&self, duration: std::time::Duration) -> #request_builder_ident #ty_generics {
                    #request_builder_ident::new(duration, self.clone())
                }
            }
        }
    }

    /// Expands the builder types.
    fn expand_builders(&self) -> TokenStream {
        let Self {
            args,
            item_impl,
            message_handlers,
            request_handlers,
            message_builder_ident,
            request_builder_ident,
            ..
        } = self;
        let vis = &args.visibility;
        let (impl_generics, ty_generics, where_clause) = item_impl.generics.split_for_impl();
        let self_ty = &item_impl.self_ty;
        let arg_phantom = if !item_impl.generics.params.is_empty() {
            Some(quote! { std::marker::PhantomData, })
        } else {
            None
        };

        let message_builder_methods = message_handlers
            .iter()
            .map(HandlerStructure::from_handler)
            .map(|handler| {
                let HandlerStructure {
                    attrs,
                    ident,
                    args,
                    message_type,
                    handler_args,
                    ..
                } = handler;

                quote! {
                    #( #attrs )*
                    #vis fn #ident(&self #(, #args )*) {
                        use lunatic::process::Message;
                        let msg = #message_type(#arg_phantom #( #handler_args ),*);
                        self.process_ref.send_after(msg, self.duration);
                    }
                }
            });

        let request_builder_methods = request_handlers
            .iter()
            .map(HandlerStructure::from_handler)
            .map(|handler| {
                let HandlerStructure {
                    attrs,
                    ident,
                    args,
                    return_ty,
                    message_type,
                    handler_args,
                    ..
                } = handler;

                quote! {
                    #( #attrs )*
                    #vis fn #ident(&self #(, #args )*) -> lunatic::MailboxResult<#return_ty> {
                        use lunatic::process::Request;
                        let req = #message_type(#arg_phantom #( #handler_args ),*);
                        self.process_ref.request_timeout(req, self.duration)
                    }
                }
            });

        quote! {
            #vis struct #message_builder_ident #ty_generics #where_clause {
                duration: std::time::Duration,
                process_ref: lunatic::process::ProcessRef<#self_ty>,
            }

            impl #impl_generics #message_builder_ident #ty_generics #where_clause {
                fn new(duration: std::time::Duration, process_ref: ProcessRef<#self_ty>) -> Self {
                    Self {
                        duration,
                        process_ref,
                    }
                }

                #( #message_builder_methods )*
            }

            #vis struct #request_builder_ident #ty_generics #where_clause {
                duration: std::time::Duration,
                process_ref: lunatic::process::ProcessRef<#self_ty>,
            }

            impl #impl_generics #request_builder_ident #ty_generics #where_clause {
                fn new(duration: std::time::Duration, process_ref: ProcessRef<#self_ty>) -> Self {
                    Self {
                        duration,
                        process_ref,
                    }
                }

                #( #request_builder_methods )*
            }
        }
    }

    /// Creates an ident for a handler ident.
    fn handler_wrapper_ident(ident: impl ToString) -> syn::Ident {
        format_ident!("__MsgWrap{}", ident.to_string().to_case(Case::Pascal))
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

enum ItemAttr {
    Init,
    Terminate,
    HandleLinkTrapped,
    HandleMessage,
    HandleRequest,
}

impl ItemAttr {
    fn from_str(s: &str) -> Option<ItemAttr> {
        match s {
            "init" => Some(ItemAttr::Init),
            "terminate" => Some(ItemAttr::Terminate),
            "handle_link_trapped" => Some(ItemAttr::HandleLinkTrapped),
            "handle_message" => Some(ItemAttr::HandleMessage),
            "handle_request" => Some(ItemAttr::HandleRequest),
            _ => None,
        }
    }
}

fn filter_typed_args<'a>(
    args: impl Iterator<Item = &'a syn::FnArg>,
) -> impl Iterator<Item = &'a syn::PatType> {
    args.filter_map(|input| match input {
        syn::FnArg::Receiver(_) => None,
        syn::FnArg::Typed(pat_type) => Some(pat_type),
    })
}

fn filter_typed_arg_names<'a>(
    args: impl Iterator<Item = &'a syn::FnArg> + 'a,
) -> impl Iterator<Item = (syn::Ident, &'a syn::Type)> + 'a {
    filter_typed_args(args)
        .enumerate()
        .map(|(i, arg)| match &*arg.pat {
            syn::Pat::Ident(pat_ident) => (pat_ident.ident.clone(), &*arg.ty),
            _ => (format_ident!("__arg_{i}"), &*arg.ty),
        })
}

struct HandlerStructure<'a> {
    attrs: &'a Vec<syn::Attribute>,
    ident: &'a syn::Ident,
    generics: &'a syn::Generics,
    args: Vec<TokenStream>,
    return_ty: syn::Type,
    message_type: syn::Ident,
    handler_args: Vec<syn::Ident>,
}

impl<'a> HandlerStructure<'a> {
    fn from_handler(handler: &'a syn::ImplItemMethod) -> Self {
        let syn::ImplItemMethod { attrs, sig, .. } = handler;
        let syn::Signature {
            ident,
            inputs,
            generics,
            output,
            ..
        } = &sig;
        let args = filter_typed_arg_names(inputs.iter())
            .map(|(ident, ty)| quote! { #ident: #ty })
            .collect();
        let return_ty = match output {
            syn::ReturnType::Default => syn::parse_quote! { () },
            syn::ReturnType::Type(_, ty) => *ty.clone(),
        };
        let message_type = AbstractProcess::handler_wrapper_ident(ident);
        let handler_args = filter_typed_arg_names(inputs.iter())
            .map(|(ident, _ty)| ident)
            .collect();

        HandlerStructure {
            attrs,
            ident,
            generics,
            args,
            return_ty,
            message_type,
            handler_args,
        }
    }
}
