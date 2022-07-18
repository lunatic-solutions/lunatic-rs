use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::ImplItem::Method;

pub(crate) fn render_abstract_process(impl_block: syn::ItemImpl) -> TokenStream {
    let impl_attrs = impl_block.attrs;

    let impl_ty = *impl_block.self_ty;
    let impl_ty_name = match &impl_ty {
        syn::Type::Path(p) => p.path.get_ident().map(|i| i.to_string()),
        _ => None,
    };

    // impl blocks for ProcessMessage and ProcessRequest
    let mut process_impls: Vec<TokenStream> = vec![];
    // AbstractProcess methods
    let mut init_impl: Option<TokenStream> = None; // contains the State type
    let mut terminate: Option<TokenStream> = None;
    let mut handle_link_trapped: Option<TokenStream> = None;
    // other items that will be left unchanged in the impl block
    let mut skipped_items: Vec<TokenStream> = vec![];
    // wrapper functions
    let mut handler_func_defs: Vec<TokenStream> = vec![];
    let mut handler_func_impls: Vec<TokenStream> = vec![];

    for item in &impl_block.items {
        match item {
            Method(method) if method.has_tag("init") => init_impl = Some(render_init(method)),
            Method(method) if method.has_tag("terminate") => {
                terminate = Some(render_terminate(method));
            }
            Method(method) if method.has_tag("handle_link_trapped") => {
                handle_link_trapped = Some(render_handle_link_trapped(method));
            }
            Method(method) if method.has_tag("process_message") => {
                let (process_imp, handle_func_def, handle_func_impl) =
                    render_process_message(method, &impl_ty);
                process_impls.push(process_imp);
                handler_func_defs.push(handle_func_def);
                handler_func_impls.push(handle_func_impl);
                skipped_items.push(quote! { #method });
            }
            Method(method) if method.has_tag("process_request") => {
                let (process_imp, handle_func_def, handle_func_impl) =
                    render_process_request(method, &impl_ty);
                process_impls.push(process_imp);
                handler_func_defs.push(handle_func_def);
                handler_func_impls.push(handle_func_impl);
                skipped_items.push(quote! { #method });
            }
            _ => {
                skipped_items.push(quote! { #item });
            }
        }
    }

    // The extra clone will be optimized away by the compiler so we can write more concise code
    let terminate_impl = terminate.clone().map(|_| {
        quote! {
            fn terminate(state: Self::State) {
                state.terminate()
            }
        }
    });

    let handle_link_trapped_impl = handle_link_trapped.clone().map(|_| {
        quote! {
            fn handle_link_trapped(state: &mut Self::State, tag: Tag) {
                state.handle_link_trapped(tag);
            }
        }
    });

    let handler_trait = format!("{}Handler", impl_ty_name.unwrap());
    let handler_trait = proc_macro2::Ident::new(&handler_trait, Span::call_site());

    quote! {
        #(#impl_attrs)*
        impl #impl_ty {
            #terminate
            #handle_link_trapped
            #(#skipped_items)*
        }

        impl lunatic::process::AbstractProcess for #impl_ty {
            type State = #impl_ty;
            #init_impl
            #terminate_impl
            #handle_link_trapped_impl
        }

        #(#process_impls)*

        trait #handler_trait {
            #(#handler_func_defs)*
        }

        impl #handler_trait for lunatic::process::ProcessRef<#impl_ty> {
            #(#handler_func_impls)*
        }
    }
}

fn render_init(item: &syn::ImplItemMethod) -> TokenStream {
    let attrs = item.attrs.iter().filter(|attr| !attr.path.is_ident("init"));

    let sig = &item.sig;
    // ensure function name is init
    let ident = if sig.ident != "init" {
        quote_spanned! {
            sig.ident.span() => compile_error!("Invalid function signature. Function name must be `init`.");
        }
    } else {
        let ident = &sig.ident;
        quote! { #ident }
    };

    let arg_type = if let syn::FnArg::Typed(arg) = sig.inputs.last().unwrap() {
        &arg.ty
    } else {
        unreachable!()
    };
    let func_args = &sig.inputs;
    let block = &item.block;

    quote! {
        type Arg = #arg_type;

        #(#attrs)*
        fn #ident(#func_args) -> Self::State #block
    }
}

fn render_terminate(item: &syn::ImplItemMethod) -> TokenStream {
    let sig = &item.sig;
    // ensure function name is init
    let ident = if sig.ident != "terminate" {
        quote_spanned! {
            sig.ident.span() => compile_error!("Invalid function signature. Function name must be `terminate`.");
        }
    } else {
        let ident = &sig.ident;
        quote! { #ident }
    };

    let self_arg = &sig.inputs;
    let block = &item.block;

    quote! {
        fn #ident(#self_arg) #block
    }
}

fn render_handle_link_trapped(item: &syn::ImplItemMethod) -> TokenStream {
    let sig = &item.sig;
    // ensure function name is init
    let ident = if sig.ident != "handle_link_trapped" {
        quote_spanned! {
            sig.ident.span() => compile_error!("Invalid function signature. Function name must be `handle_link_trapped`.");
        }
    } else {
        let ident = &sig.ident;
        quote! { #ident }
    };

    let self_arg = &sig.inputs;
    let block = &item.block;

    quote! {
        fn #ident(#self_arg) #block
    }
}

fn render_process_message(
    item: &syn::ImplItemMethod,
    ident: &syn::Type,
) -> (TokenStream, TokenStream, TokenStream) {
    let attrs = item
        .attrs
        .iter()
        .filter(|attr| !attr.path.is_ident("process_message"));

    let sig = &item.sig;
    let fn_ident = &sig.ident;
    let message_type = match sig.inputs.last().unwrap() {
        syn::FnArg::Typed(arg) => {
            let ty = &arg.ty;
            quote! { #ty }
        }
        _ => unreachable!(),
    };

    (
        quote! {
            #(#attrs)*
            impl lunatic::process::ProcessMessage<#message_type> for #ident {
                fn handle(state: &mut Self::State, message: #message_type) {
                    state.#fn_ident(message)
                }
            }
        },
        quote! {
            fn #fn_ident(&self, message: #message_type);
        },
        quote! {
            fn #fn_ident(&self, message: #message_type) {
                self.send(message);
            }
        },
    )
}

fn render_process_request(
    item: &syn::ImplItemMethod,
    ident: &syn::Type,
) -> (TokenStream, TokenStream, TokenStream) {
    let attrs = item
        .attrs
        .iter()
        .filter(|attr| !attr.path.is_ident("process_request"));

    let sig = &item.sig;
    let fn_ident = &sig.ident;
    let message_type = match sig.inputs.last().unwrap() {
        syn::FnArg::Typed(arg) => {
            let ty = &arg.ty;
            quote! { #ty }
        }
        _ => unreachable!(),
    };
    let response_type = match &sig.output {
        syn::ReturnType::Type(_, ty) => quote! { #ty },
        syn::ReturnType::Default => {
            quote! { () }
        }
    };

    (
        quote! {
            #(#attrs)*
            impl lunatic::process::ProcessRequest<#message_type> for #ident {
                type Response = #response_type;
                fn handle(state: &mut Self::State, request: #message_type) -> #response_type {
                    state.#fn_ident(request)
                }
            }
        },
        quote! {
            fn #fn_ident(&self, message: #message_type) -> #response_type;
        },
        quote! {
            fn #fn_ident(&self, message: #message_type) -> #response_type {
                self.request(message)
            }
        },
    )
}

trait HasTag {
    fn has_tag(&self, tag: &str) -> bool;
}

impl HasTag for syn::ImplItemMethod {
    fn has_tag(&self, tag: &str) -> bool {
        self.attrs.iter().any(|attr| attr.path.is_ident(tag))
    }
}
