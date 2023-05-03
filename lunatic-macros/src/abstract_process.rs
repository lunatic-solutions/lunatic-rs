use std::iter::repeat;

use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{FnArg, PathArguments, Token, Type};

pub struct AbstractProcess {
    /// Arguments passed to the `#[abstract_process(...)]` macro.
    args: Args,
    /// Original impl item.
    item_impl: syn::ItemImpl,
    /// Arg type in abstract process implementation.
    arg_ty: syn::Type,
    /// `init` method.
    init: syn::ImplItemMethod,
    /// Terminate method.
    terminate: Option<syn::ImplItemMethod>,
    /// Handle link died method.
    handle_link_death: Option<syn::ImplItemMethod>,
    /// Message handler methods.
    message_handlers: Vec<syn::ImplItemMethod>,
    /// Request handler methods.
    request_handlers: Vec<syn::ImplItemMethod>,
    /// Deferred request handler methods.
    deferred_request_handlers: Vec<syn::ImplItemMethod>,
    /// Name of trait wrapping messages
    message_trait_name: syn::Ident,
    /// Name of trait wrapping requests
    request_trait_name: syn::Ident,
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
        let (
            init,
            terminate,
            handle_link_death,
            message_handlers,
            request_handlers,
            deferred_request_handlers,
        ) = item_impl
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
                Ok((None, None, None, Vec::new(), Vec::new(), Vec::new())),
                |acc, (item_attr, impl_item_method)| {
                    let (
                        mut init,
                        mut terminate,
                        mut handle_link_death,
                        mut message_handlers,
                        mut request_handlers,
                        mut deferred_request_handlers,
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
                            if handle_link_death.is_some() {
                                return Err(syn::Error::new(
                                    impl_item_method.sig.ident.span(),
                                    "handle_link_death method already defined",
                                ));
                            }

                            handle_link_death = Some(impl_item_method);
                        }
                        ItemAttr::HandleMessage => {
                            message_handlers.push(impl_item_method);
                        }
                        ItemAttr::HandleRequest => {
                            request_handlers.push(impl_item_method);
                        }
                        ItemAttr::HandleDeferredRequest => {
                            deferred_request_handlers.push(impl_item_method);
                        }
                    }

                    Ok((
                        init,
                        terminate,
                        handle_link_death,
                        message_handlers,
                        request_handlers,
                        deferred_request_handlers,
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

        let message_trait_name = args
            .message_trait_name
            .as_ref()
            .map(|message_trait_name| format_ident!("{}", message_trait_name.value()))
            .unwrap_or_else(|| format_ident!("{}Messages", self_ident));
        let request_trait_name = args
            .request_trait_name
            .as_ref()
            .map(|request_trait_name| format_ident!("{}", request_trait_name.value()))
            .unwrap_or_else(|| format_ident!("{}Requests", self_ident));

        Ok(AbstractProcess {
            args,
            item_impl,
            arg_ty,
            init,
            terminate,
            handle_link_death,
            message_handlers,
            request_handlers,
            deferred_request_handlers,
            message_trait_name,
            request_trait_name,
        })
    }

    /// Expands macro.
    pub fn expand(&self) -> TokenStream {
        let handler_wrappers = self.expand_handler_wrappers();
        let original_impl = self.expand_original_impl();
        let impl_abstract_process = self.expand_impl_abstract_process();
        let message_handler_impls = self.expand_message_handler_impls();
        let request_handler_impls = self.expand_request_handler_impls();
        let deferred_request_handler_impls = self.expand_deferred_request_handler_impls();
        let handler_trait = self.expand_handler_trait();
        let impl_handler_trait = self.expand_impl_handler_trait();

        quote! {
            #handler_wrappers
            #original_impl
            #impl_abstract_process
            #message_handler_impls
            #request_handler_impls
            #deferred_request_handler_impls
            #handler_trait
            #impl_handler_trait
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
            .map(|impl_item_method| self.expand_handler_wrapper(impl_item_method, false));

        // Exclude last element that is a `DeferredResponse`
        let dr_wrappers = self
            .deferred_request_handlers
            .iter()
            .map(|impl_item_method| self.expand_handler_wrapper(impl_item_method, true));
        quote! {
            #( #wrappers )*
            #( #dr_wrappers )*
        }
    }

    /// Expands a single handler wrapper struct.
    ///
    /// ```ignore
    /// __MsgWrap(Param1, Param2);
    /// ```
    fn expand_handler_wrapper(
        &self,
        impl_item_method: &syn::ImplItemMethod,
        exclude_last: bool,
    ) -> TokenStream {
        let vis = &self.args.visibility;
        let ident = Self::handler_wrapper_ident(&impl_item_method.sig.ident);
        let (_, ty_generics, _) = &self.item_impl.generics.split_for_impl();
        let phantom_generics = &self.item_impl.generics.params;
        let inputs = match exclude_last {
            true => {
                // Exclude last element.
                let mut a: Vec<FnArg> = impl_item_method.sig.inputs.clone().into_iter().collect();
                a.pop();
                a
            }
            false => impl_item_method.sig.inputs.clone().into_iter().collect(),
        };
        let fields = filter_typed_args(inputs.iter()).map(|field| &*field.ty);
        let phantom_field = if !self.item_impl.generics.params.is_empty() {
            Some(quote! { std::marker::PhantomData <(#phantom_generics)>, })
        } else {
            None
        };

        quote! {
            #[derive(serde::Serialize, serde::Deserialize)]
            #vis struct #ident #ty_generics (
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
        let serializer = match &self.args.serializer {
            Some(serializer) => quote!(#serializer),
            None => quote!(lunatic::serializer::Bincode),
        };
        let handlers = self.expand_type_handlers();

        let (init_impl, startup_error) = self.expand_init_impl();
        let terminate_impl = self.expand_terminate_impl();
        let handle_link_death_impl = self.expand_handle_link_death_impl();

        quote! {
            impl #impl_generics lunatic::ap::AbstractProcess for #self_ty #where_clause {
                type State = #self_ty;
                type Arg = #arg_ty;
                type Serializer = #serializer;
                type Handlers = (#handlers);
                type StartupError = #startup_error;

                #init_impl
                #terminate_impl
                #handle_link_death_impl
            }
        }
    }

    /// Collects all wrapper types and adds them to the `AP::Handlers` tuple.
    fn expand_type_handlers(&self) -> TokenStream {
        let message_wrappers = self.message_handlers.iter().map(|impl_item_method| {
            let ident = Self::handler_wrapper_ident(&impl_item_method.sig.ident);
            let (_, generics, _) = &self.item_impl.generics.split_for_impl();
            quote! { lunatic::ap::handlers::Message<#ident #generics>, }
        });
        let request_wrappers = self.request_handlers.iter().map(|impl_item_method| {
            let ident = Self::handler_wrapper_ident(&impl_item_method.sig.ident);
            let (_, generics, _) = &self.item_impl.generics.split_for_impl();
            quote! { lunatic::ap::handlers::Request<#ident #generics>, }
        });
        let deferred_request_wrappers =
            self.deferred_request_handlers
                .iter()
                .map(|impl_item_method| {
                    let ident = Self::handler_wrapper_ident(&impl_item_method.sig.ident);
                    let (_, generics, _) = &self.item_impl.generics.split_for_impl();
                    quote! { lunatic::ap::handlers::DeferredRequest<#ident #generics>, }
                });

        message_wrappers
            .chain(request_wrappers)
            .chain(deferred_request_wrappers)
            .collect()
    }

    /// Expands the `init` method in the abstract process implementation.
    fn expand_init_impl(&self) -> (TokenStream, TokenStream) {
        let ident = &self.init.sig.ident;
        let arg_ty = &self.arg_ty;

        let init = quote! {
            fn init(config: lunatic::ap::Config<Self>, arg: #arg_ty) -> Result<Self::State, Self::StartupError> {
                Self::#ident(config, arg)
            }
        };

        // Extract startup error type from `Result<T, Error>`.
        let startup_error = match &self.init.sig.output {
            syn::ReturnType::Type(_, ret_type) => {
                match ret_type.as_ref() {
                    Type::Path(ret_type) => match ret_type.path.segments.last() {
                        Some(ret_type_segments) => match &ret_type_segments.arguments {
                            PathArguments::AngleBracketed(generics) => {
                                // Error is the last argument in `Result<T, Error>`.
                                let last = generics.args.last();
                                quote! { #last }
                            }
                            _ => quote! { () },
                        },
                        _ => quote! { () },
                    },
                    _ => quote! { () },
                }
            }
            _ => quote! { () },
        };
        (init, startup_error)
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

    /// Expands the `handle_link_death` method in the abstract process
    /// implementation.
    fn expand_handle_link_death_impl(&self) -> TokenStream {
        self.handle_link_death
            .as_ref()
            .map(|handle_link_death| {
                let ident = &handle_link_death.sig.ident;

                quote! {
                    fn handle_link_death(mut state: lunatic::ap::State<Self>, tag: lunatic::Tag) {
                        state.#ident(tag);
                    }
                }
            })
            .unwrap_or_default()
    }

    /// Expands the `MessageHandler` implementations for the message handler
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
            let offset = usize::from(!self.item_impl.generics.params.is_empty());
            let message_fields = (offset..args.count() + offset).map(|i| {
                let i = proc_macro2::Literal::usize_unsuffixed(i);
                quote! { message. #i }
            });

            quote! {
                #( #attrs )*
                impl #impl_generics lunatic::ap::MessageHandler<#message_type #ty_generics> for #self_ty #where_clause {
                    fn handle(mut state: lunatic::ap::State<Self>, message: #message_type #ty_generics) {
                        state.#fn_ident(#( #message_fields ),*)
                    }
                }
            }
        });

        quote! {
            #( #message_handler_impls )*
        }
    }

    /// Expands the `RequestHandler` implementations for the request handler
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
            let offset = usize::from(!self.item_impl.generics.params.is_empty());
            let request_fields = (offset..args.count() + offset).map(|i| {
                let i = proc_macro2::Literal::usize_unsuffixed(i);
                quote! { request. #i }
            });

            quote! {
                #( #attrs )*
                impl #impl_generics lunatic::ap::RequestHandler<#request_type #ty_generics> for #self_ty #where_clause {
                    type Response = #response_type;

                    fn handle(mut state: lunatic::ap::State<Self>, request: #request_type #ty_generics) -> Self::Response {
                        state.#fn_ident(#( #request_fields ),*)
                    }
                }
            }
        });

        quote! {
            #( #request_handler_impls )*
        }
    }

    /// Expands the `DeferredRequestHandler` implementations for the deferred
    /// request handler wrapper types.
    fn expand_deferred_request_handler_impls(&self) -> TokenStream {
        let request_handler_impls = self.deferred_request_handlers.iter().map(|request_handler| {
            let syn::ImplItemMethod {
                attrs,
                sig,
                ..
            } = request_handler;
            let self_ty = &self.item_impl.self_ty;
            let request_type = Self::handler_wrapper_ident(&sig.ident);
            // Get the first generic of the last argument `DeferredRequest<THIS, _>`.
            let response_type = match &sig.inputs.last() {
                Some(FnArg::Typed(path)) => match &*path.ty {
                        Type::Path(path) => {
                            let last = path.path.segments.last().unwrap();
                            match &last.arguments {
                                PathArguments::AngleBracketed(generics) => {
                                    let response_type = generics.args.first().unwrap();
                                    quote!{ #response_type }
                                },
                                _ => quote!{()},
                            }
                        },
                        _ => quote!{()},
                },
                _ => quote!{()},
            };
            let fn_ident = &sig.ident;
            let (impl_generics, ty_generics, where_clause) = self.item_impl.generics.split_for_impl();
            let args = filter_typed_args(sig.inputs.iter());
            let offset = usize::from(!self.item_impl.generics.params.is_empty());
            // Exclude last argument
            let request_fields = (offset..args.count() - 1 + offset).map(|i| {
                let i = proc_macro2::Literal::usize_unsuffixed(i);
                quote! { request. #i }
            });

            quote! {
                #( #attrs )*
                impl #impl_generics lunatic::ap::DeferredRequestHandler<#request_type #ty_generics> for #self_ty #where_clause {
                    type Response = #response_type;

                    fn handle(
                        mut state: lunatic::ap::State<Self>,
                        request: #request_type #ty_generics,
                        deferred_response: lunatic::ap::DeferredResponse<Self::Response, Self>) {                        
                            state.#fn_ident(#( #request_fields, )* deferred_response);
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
            deferred_request_handlers,
            message_trait_name,
            request_trait_name,
            ..
        } = self;
        let vis = &args.visibility;
        let (_impl_generics, ty_generics, where_clause) = item_impl.generics.split_for_impl();

        let message_handler_defs = message_handlers
            .iter()
            .zip(repeat(false)) // is_deferred = false
            .map(HandlerStructure::from_handler)
            .map(|handler| {
                let HandlerStructure {
                    attrs,
                    ident,
                    generics,
                    args,
                    ..
                } = handler;

                let return_ty_type = format_ident!("ReturnTy_{}", ident);
                quote! {
                    #[allow(non_camel_case_types)]
                    type #return_ty_type;
                    #( #attrs )*
                    fn #ident #generics (&self #(, #args )*) -> Self::#return_ty_type;
                }
            });

        let request_handler_defs = request_handlers
            .iter()
            .zip(repeat(false)) // is_deferred = false
            .map(HandlerStructure::from_handler)
            .map(|handler| {
                let HandlerStructure {
                    attrs,
                    ident,
                    generics,
                    args,
                    ..
                } = handler;

                let return_ty_type = format_ident!("ReturnTy_{}", ident);
                quote! {
                    #[allow(non_camel_case_types)]
                    type #return_ty_type;
                    #( #attrs )*
                    fn #ident #generics (&self #(, #args )*) -> Self::#return_ty_type;
                }
            });

        let deferred_request_handler_defs = deferred_request_handlers
            .iter()
            .zip(repeat(true)) // is_deferred = true
            .map(HandlerStructure::from_handler)
            .map(|handler| {
                let HandlerStructure {
                    attrs,
                    ident,
                    generics,
                    mut args,
                    ..
                } = handler;

                // Remove last argument from input.
                args.pop();
                let return_ty_type = format_ident!("ReturnTy_{}", ident);
                quote! {
                    #[allow(non_camel_case_types)]
                    type #return_ty_type;
                    #( #attrs )*
                    fn #ident #generics (&self #(, #args )*) -> Self::#return_ty_type;
                }
            });

        quote! {
            #vis trait #message_trait_name #ty_generics #where_clause {
                #( #message_handler_defs )*
            }

            #vis trait #request_trait_name #ty_generics #where_clause {
                #( #request_handler_defs )*
                #( #deferred_request_handler_defs )*
            }
        }
    }

    /// Expands the implementation of the `Handler` trait.
    fn expand_impl_handler_trait(&self) -> TokenStream {
        let Self {
            item_impl,
            message_handlers,
            request_handlers,
            deferred_request_handlers,
            message_trait_name,
            request_trait_name,
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
            .zip(repeat(false)) // is_deferred = false
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

                let return_ty_type = format_ident!("ReturnTy_{}", ident);
                quote! {
                    type #return_ty_type = ();
                    #( #attrs )*
                    fn #ident #generics (&self #(, #args )*) {
                        let msg = #message_type(#arg_phantom #( #handler_args ),*);
                        self.send(msg);
                    }
                }
            });

        let message_delay_handler_impls = message_handlers
            .iter()
            .zip(repeat(false)) // is_deferred = false
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

                let return_ty_type = format_ident!("ReturnTy_{}", ident);
                quote! {
                    type #return_ty_type = lunatic::time::TimerRef;
                    #( #attrs )*
                    fn #ident #generics (&self #(, #args )*) -> lunatic::time::TimerRef {
                        let msg = #message_type(#arg_phantom #( #handler_args ),*);
                        self.send(msg)
                    }
                }
            });

        let request_handler_impls = request_handlers
            .iter()
            .zip(repeat(false)) // is_deferred = false
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

                let return_ty_type = format_ident!("ReturnTy_{}", ident);
                quote! {
                    type #return_ty_type = #return_ty;
                    #( #attrs )*
                    fn #ident #generics (&self #(, #args )*) -> Self::#return_ty_type {
                        let req = #message_type(#arg_phantom #( #handler_args ),*);
                        self.request(req)
                    }
                }
            });

        let request_timeout_handler_impls = request_handlers
            .iter()
            .zip(repeat(false)) // is_deferred = false
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

                let return_ty_type = format_ident!("ReturnTy_{}", ident);
                quote! {
                    type #return_ty_type = Result<#return_ty, lunatic::time::Timeout>;
                    #( #attrs )*
                    fn #ident #generics (&self #(, #args )*) -> Self::#return_ty_type {
                        let req = #message_type(#arg_phantom #( #handler_args ),*);
                        self.request(req)
                    }
                }
            });

        let deferred_request_handler_impls = deferred_request_handlers
            .iter()
            .zip(repeat(true)) // is_deferred = true
            .map(HandlerStructure::from_handler)
            .map(|handler| {
                let HandlerStructure {
                    attrs,
                    ident,
                    generics,
                    mut args,
                    return_ty,
                    message_type,
                    mut handler_args,
                } = handler;

                // Remove last argument from input.
                args.pop();
                handler_args.pop();
                let return_ty_type = format_ident!("ReturnTy_{}", ident);
                quote! {
                    type #return_ty_type = #return_ty;
                    #( #attrs )*
                    fn #ident #generics (&self #(, #args )*) -> Self::#return_ty_type {
                        let req = #message_type(#arg_phantom #( #handler_args ),*);
                        self.deferred_request(req)
                    }
                }
            });

        let deferred_request_timeout_handler_impls = deferred_request_handlers
            .iter()
            .zip(repeat(true)) // is_deferred = true
            .map(HandlerStructure::from_handler)
            .map(|handler| {
                let HandlerStructure {
                    attrs,
                    ident,
                    generics,
                    mut args,
                    return_ty,
                    message_type,
                    mut handler_args,
                } = handler;

                // Remove last argument from input.
                args.pop();
                handler_args.pop();
                let return_ty_type = format_ident!("ReturnTy_{}", ident);
                quote! {
                    type #return_ty_type = Result<#return_ty, lunatic::time::Timeout>;
                    #( #attrs )*
                    fn #ident #generics (&self #(, #args )*) -> Self::#return_ty_type {
                        let req = #message_type(#arg_phantom #( #handler_args ),*);
                        self.deferred_request(req)
                    }
                }
            });

        quote! {
            impl #impl_generics #message_trait_name #ty_generics for lunatic::ap::ProcessRef<#self_ty> #where_clause {
                #( #message_handler_impls )*
            }

            impl #impl_generics #request_trait_name #ty_generics for lunatic::ap::ProcessRef<#self_ty> #where_clause {
                #( #request_handler_impls )*
                #( #deferred_request_handler_impls )*
            }

            impl #impl_generics #message_trait_name #ty_generics for
                    lunatic::time::WithDelay<lunatic::ap::ProcessRef<#self_ty>> #where_clause {
                #( #message_delay_handler_impls )*
            }

            impl #impl_generics #request_trait_name #ty_generics for
                    lunatic::time::WithTimeout<lunatic::ap::ProcessRef<#self_ty>> #where_clause {
                #( #request_timeout_handler_impls )*
                #( #deferred_request_timeout_handler_impls )*
            }
        }
    }

    /// Create a wrapper name for the request and send
    fn handler_wrapper_ident(ident: impl ToString) -> syn::Ident {
        format_ident!("__MsgWrap{}", ident.to_string().to_case(Case::Pascal))
    }
}

#[derive(Default)]
pub struct Args {
    message_trait_name: Option<syn::LitStr>,
    request_trait_name: Option<syn::LitStr>,
    visibility: Option<syn::Visibility>,
    serializer: Option<syn::Type>,
}

impl Args {
    fn parse_arg(&mut self, input: ParseStream) -> syn::Result<()> {
        if input.is_empty() {
            return Ok(());
        }

        let ident: syn::Ident = input.parse()?;
        let _: syn::Token![=] = input.parse()?;
        if ident == "message_trait_name" {
            if self.message_trait_name.is_some() {
                return Err(syn::Error::new(
                    ident.span(),
                    "message trait name already specified",
                ));
            }

            self.message_trait_name = Some(input.parse()?);
        } else if ident == "request_trait_name" {
            if self.request_trait_name.is_some() {
                return Err(syn::Error::new(
                    ident.span(),
                    "request trait name already specified",
                ));
            }

            self.request_trait_name = Some(input.parse()?);
        } else if ident == "visibility" {
            if self.visibility.is_some() {
                return Err(syn::Error::new(
                    ident.span(),
                    "visibility already specified",
                ));
            }

            self.visibility = Some(input.parse()?);
        } else if ident == "serializer" {
            if self.serializer.is_some() {
                return Err(syn::Error::new(
                    ident.span(),
                    "serializer already specified",
                ));
            }

            self.serializer = Some(input.parse()?);
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
    HandleDeferredRequest,
}

impl ItemAttr {
    fn from_str(s: &str) -> Option<ItemAttr> {
        match s {
            "init" => Some(ItemAttr::Init),
            "terminate" => Some(ItemAttr::Terminate),
            "handle_link_death" => Some(ItemAttr::HandleLinkTrapped),
            "handle_message" => Some(ItemAttr::HandleMessage),
            "handle_request" => Some(ItemAttr::HandleRequest),
            "handle_deferred_request" => Some(ItemAttr::HandleDeferredRequest),
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
    return_ty: TokenStream,
    message_type: syn::Ident,
    handler_args: Vec<syn::Ident>,
}

impl<'a> HandlerStructure<'a> {
    fn from_handler((handler, is_deferred): (&'a syn::ImplItemMethod, bool)) -> Self {
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
        let return_ty = if is_deferred {
            match &inputs.last() {
                Some(FnArg::Typed(path)) => match &*path.ty {
                    Type::Path(path) => {
                        let last = path.path.segments.last().unwrap();
                        match &last.arguments {
                            PathArguments::AngleBracketed(generics) => {
                                let response_type = generics.args.first().unwrap();
                                quote! { #response_type }
                            }
                            _ => quote! {()},
                        }
                    }
                    _ => quote! {()},
                },
                _ => quote! {()},
            }
        } else {
            match output {
                syn::ReturnType::Default => quote! {()},
                syn::ReturnType::Type(_, ty) => quote! {#ty},
            }
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
